package groupcachepb

// import "google.golang.org/protobuf/proto"

type GetRequest struct {
	Group            *string `protobuf:"bytes,1,req,name=group" json:"group,omitempty"`
	Key              *string `protobuf:"bytes,2,req,name=key" json:"key,omitempty"`
	XXX_unrecognized []byte  `json:"-"`
}

type GetResponse struct {
	Value            []byte   `protobuf:"bytes,1,opt,name=value" json:"value,omitempty"`
	MinuteQps        *float64 `protobuf:"fixed64,2,opt,name=minute_qps" json:"minute_qps,omitempty"`
	XXX_unrecognized []byte   `json:"-"`
}
