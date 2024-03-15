package minigroupcache

type Sink interface {
	SetString(s string) error
	SetBytes(v []byte) error
	View() (ByteView, error)
}

func setSinkView(s Sink, v ByteView) error {
	type viewSetter interface {
		setView(v ByteView) error
	}
	if vs, ok := s.(viewSetter); ok {
		return vs.setView(v)
	}
	if v.b != nil {
		return s.SetBytes(v.b)
	}
	return s.SetString(v.s)
}

func ByteViewSink(dst *ByteView) Sink {
	if dst == nil {
		panic("")
	}
	return &byteViewSink{dst: dst}
}

type byteViewSink struct {
	dst *ByteView
}

func (s *byteViewSink) SetString(v string) error {
	*s.dst = ByteView{s: v}
	return nil
}

func (s *byteViewSink) SetBytes(b []byte) error {
	*s.dst = ByteView{b: cloneBytes(b)}
	return nil
}

func (s *byteViewSink) View() (ByteView, error) {
	return *s.dst, nil
}

func (s *byteViewSink) setView(v ByteView) error {
	*s.dst = v
	return nil
}
