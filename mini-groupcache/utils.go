package minigroupcache

import (
	"strconv"
	"sync/atomic"
)

func cloneBytes(b []byte) []byte {
	c := make([]byte, len(b))
	copy(c, b)
	return c
}

type AtomicInt int64

func (i *AtomicInt) Add(n int64) {
	atomic.AddInt64((*int64)(i), n)
}

func (i *AtomicInt) Get() int64 {
	return atomic.LoadInt64((*int64)(i))
}

func (i *AtomicInt) String() string {
	return strconv.FormatInt(i.Get(), 10)
}
