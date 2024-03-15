package minigroupcache

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"mini-groupcache/consistenthash"
	"mini-groupcache/groupcachepb"
	"net/http"
	"net/url"
	"strings"
	"sync"
	// "google.golang.org/protobuf/proto"
)

const (
	defaultReplicas = 50
	defaultBasePath = "/groupcache/"
)

type HTTPPool struct {
	Context     func(*http.Request) context.Context
	Transport   func(context.Context) http.RoundTripper
	self        string
	opts        HTTPPollOptions
	mu          sync.Mutex
	peers       *consistenthash.Map
	httpGetters map[string]*httpGetter
}

type HTTPPollOptions struct {
	BasePath string
	Replicas int
	HashFn   consistenthash.Hash
}

func NewHTTPPool(self string) *HTTPPool {
	p := NewHTTPPoolOpts(self, nil)
	return p
}

func NewHTTPPoolOpts(self string, o *HTTPPollOptions) *HTTPPool {
	p := &HTTPPool{
		self: self,
		httpGetters: make(map[string]*httpGetter),
	}
	if o != nil {
		p.opts = *o
	}
	if p.opts.Replicas == 0 {
		p.opts.Replicas = defaultReplicas
	}
	if p.opts.BasePath == "" {
		p.opts.BasePath = defaultBasePath
	}
	p.peers = consistenthash.New(p.opts.Replicas, p.opts.HashFn)
	RegisterPeerPicker(func() PeerPicker { return p })
	return p
}

func (p *HTTPPool) Set(peers ...string) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.peers = consistenthash.New(p.opts.Replicas, p.opts.HashFn)
	p.peers.Add(peers...)
	p.httpGetters = make(map[string]*httpGetter, len(peers))
	for _, peer := range peers {
		p.httpGetters[peer] = &httpGetter{transport: p.Transport, baseURL: peer + p.opts.BasePath}
	}
}

func (p *HTTPPool) PickePeer(key string) (ProtoGetter, bool) {
	p.mu.Lock()
	defer p.mu.Unlock()
	if p.peers.IsEmpty() {
		return nil, false
	}
	if peer := p.peers.Get(key); peer != p.self {
		return p.httpGetters[peer], true
	}
	return nil, false
}

func (p *HTTPPool) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if !strings.HasPrefix(r.URL.Path, p.opts.BasePath) {
		panic("")
	}
	parts := strings.SplitN(r.URL.Path[len(p.opts.BasePath):], "/", 2)
	if len(parts) != 2 {
		http.Error(w, "bad request", http.StatusBadRequest)
		return
	}
	groupName := parts[0]
	// key := parts[1]
	group := GetGroup(groupName)
	if group == nil {
		http.Error(w, "no such group: "+groupName, http.StatusNotFound)
		return
	}
	
}

type httpGetter struct {
	baseURL   string
	transport func(context.Context) http.RoundTripper
}

var bufferPool = sync.Pool {
	New: func() interface{} { return new(bytes.Buffer) },
}

func (h *httpGetter) Get(ctx context.Context, in *groupcachepb.GetRequest, out *groupcachepb.GetResponse) error {
	u := fmt.Sprintf(
		"%v%v/%v",
		h.baseURL,
		url.QueryEscape(""),
		url.QueryEscape(""),
	)
	req, err := http.NewRequest("GET", u, nil)
	if err != nil {
		return err
	}
	req = req.WithContext(ctx)
	tr := http.DefaultTransport
	if h.transport != nil {
		tr = h.transport(ctx)
	}
	res, err := tr.RoundTrip(req)
	if err != nil {
		return err
	}
	defer res.Body.Close()
	if res.StatusCode != http.StatusOK {
		return fmt.Errorf("server returned: %v", res.Status)
	}
	b := bufferPool.Get().(*bytes.Buffer)
	b.Reset()
	defer bufferPool.Put(b)
	_, err = io.Copy(b, res.Body)
	if err != nil {
		return fmt.Errorf("server returned: %v", res.Status)
	}
	// err = proto.Unmarshal(b.Bytes(), out)
	if err != nil {
		return fmt.Errorf("")
	}
	return nil
}
