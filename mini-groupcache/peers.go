package minigroupcache

import (
	"context"
	"mini-groupcache/groupcachepb"
)

type ProtoGetter interface {
	Get(ctx context.Context, in *groupcachepb.GetRequest, out *groupcachepb.GetResponse) error
}

type PeerPicker interface {
	PickePeer(key string) (peer ProtoGetter, ok bool)
}

var (
	portPicker func(groupName string) PeerPicker
)

func RegisterPeerPicker(fn func() PeerPicker) {
	if portPicker != nil {
		panic("")
	}
	portPicker = func(_ string) PeerPicker { return fn() }
}

func RegisterPerGroupPeerPicker(fn func(groupName string) PeerPicker) {
	if portPicker != nil {
		panic("")
	}
	portPicker = fn
}

func getPeers(groupName string) PeerPicker {
	if portPicker == nil {
		panic("")
	}
	pk := portPicker(groupName)
	if pk == nil {
		panic("")
	}
	return pk
}
