package minigroupcache

import (
	"context"
	"errors"
	"math/rand"
	"mini-groupcache/groupcachepb"
	"mini-groupcache/singleflight"
	"sync"
)

type Getter interface {
	Get(ctx context.Context, key string, dest Sink) error
}

type GetterFunc func(ctx context.Context, key string, dest Sink) error

func (f GetterFunc) Get(ctx context.Context, key string, dest Sink) error {
	return f(ctx, key, dest)
}

var (
	mu                 sync.RWMutex
	initPeerServer     func()
	initPeerServerOnce sync.Once
	newGroupHook       func(*Group)
	groups = make(map[string]*Group)
)

func GetGroup(name string) *Group {
	mu.RLock()
	g := groups[name]
	mu.RUnlock()
	return g
}

func RegisterServerStart(fn func()) {
	if initPeerServer != nil {
		panic("")
	}
	initPeerServer = fn
}

func RegisterNewGroupHook(fn func(*Group)) {
	if newGroupHook != nil {
		panic("")
	}
	newGroupHook = fn
}

func NewGroup(name string, cacheBytes int64, getter Getter) *Group {
	return newGroup(name, cacheBytes, getter, nil)
}

func callInitPeerServer() {
	if initPeerServer != nil {
		initPeerServer()
	}
}

func newGroup(name string, cacheBytes int64, getter Getter, peers PeerPicker) *Group {
	if getter == nil {
		panic("")
	}
	mu.Lock()
	defer mu.Unlock()
	initPeerServerOnce.Do(callInitPeerServer)
	if _, dup := groups[name]; dup {
		panic("")
	}
	g := &Group{
		name:       name,
		getter:     getter,
		peers:      peers,
		cacheBytes: cacheBytes,
		loadGroup:  &singleflight.Group{},
	}
	if fn := newGroupHook; fn != nil {
		fn(g)
	}
	groups[name] = g
	return g
}

type flightGroup interface {
	Do(key string, fn func() (interface{}, error)) (interface{}, error)
}

type Group struct {
	name       string
	getter     Getter
	peers      PeerPicker
	peersOnce  sync.Once
	mainCache  cache
	hotCache   cache
	cacheBytes int64
	loadGroup  flightGroup
}

func (g *Group) initPeers() {
	if g.peers == nil {
		g.peers = getPeers(g.name)
	}
}

func (g *Group) Get(ctx context.Context, key string, dest Sink) error {
	g.peersOnce.Do(g.initPeers)
	if dest == nil {
		return errors.New("groupcache: nil dest Sink")
	}
	value, cacheHit := g.lookupCache(key)
	if cacheHit {
		return setSinkView(dest, value)
	}
	value, destPopulated, err := g.load(ctx, key, dest)
	if err != nil {
		return err
	}
	if destPopulated {
		return nil
	}
	return setSinkView(dest, value)
}

func (g *Group) load(ctx context.Context, key string, dest Sink) (value ByteView, destPopulated bool, err error) {
	viewi, err := g.loadGroup.Do(key, func() (interface{}, error) {
		if value, cacheHit := g.lookupCache(key); cacheHit {
			return value, nil
		}
		var value ByteView
		var err error
		if peer, ok := g.peers.PickePeer(key); ok {
			value, err = g.getFromPeer(ctx, peer, key)
			if err == nil {
				return value, nil
			}
		}
		value, err = g.getLocally(ctx, key, dest)
		if err != nil {
			return nil, err
		}
		g.populateCache(key, value, &g.mainCache)
		return value, nil
	})
	if err == nil {
		value = viewi.(ByteView)
	}
	return
}

func (g *Group) lookupCache(key string) (value ByteView, ok bool) {
	if g.cacheBytes <= 0 {
		return
	}
	value, ok = g.mainCache.get(key)
	if ok {
		return
	}
	value, ok = g.hotCache.get(key)
	return
}

func (g *Group) populateCache(key string, value ByteView, cache *cache) {
	if g.cacheBytes <= 0 {
		return
	}
	cache.add(key, value)
	for {
		mainBytes := g.mainCache.bytes()
		hotBytes := g.hotCache.bytes()
		if mainBytes + hotBytes <= g.cacheBytes {
			return
		}
		victim := &g.mainCache
		if hotBytes > mainBytes / 8 {
			victim = &g.hotCache
		}
		victim.removeOldest()
	}
}

func (g *Group) getLocally(ctx context.Context, key string, dest Sink) (ByteView, error) {
	err := g.getter.Get(ctx, key, dest)
	if err != nil {
		return ByteView{}, err
	}
	return dest.View()
}

func (g *Group) getFromPeer(ctx context.Context, peer ProtoGetter, key string) (ByteView, error) {
	req := &groupcachepb.GetRequest{
		Group: &g.name,
		Key: &key,
	}
	res := &groupcachepb.GetResponse{}
	err := peer.Get(ctx, req, res)
	if err != nil {
		return ByteView{}, err
	}
	value := ByteView{b: res.Value}
	if rand.Intn(10) == 0 {
		g.populateCache(key, value, &g.hotCache)
	}
	return value, nil
}
