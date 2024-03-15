package minigroupcache

type ByteView struct {
	b []byte
	s string
}

func (v ByteView) Len() int {
	if v.b != nil {
		return len(v.b)
	}
	return len(v.s)
}

func (v ByteView) Bytes() []byte {
	if v.b != nil {
		return cloneBytes(v.b)
	}
	return []byte(v.s)
}

func (v ByteView) String() string {
	if v.b != nil {
		return string(v.b)
	}
	return v.s
}
