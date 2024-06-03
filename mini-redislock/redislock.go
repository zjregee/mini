package main

import (
	"context"
	"crypto/rand"
	_ "embed"
	"encoding/base64"
	"errors"
	"io"
	"strconv"
	"sync"
	"time"

	"github.com/redis/go-redis/v9"
)

//go:embed release.lua
var luaReleaseScript string

//go:embed refresh.lua
var luaRefreshScript string

//go:embed pttl.lua
var luaPTTLScript string

//go:embed obtain.lua
var luaObtainScript string

var (
	luaRefresh = redis.NewScript(luaRefreshScript)
	luaRelease = redis.NewScript(luaReleaseScript)
	luaPTTL    = redis.NewScript(luaPTTLScript)
	luaObtain  = redis.NewScript(luaObtainScript)
)

var (
	ErrNotObtained = errors.New("redislock: not obtained")
	ErrLockNotHeld = errors.New("redislock: lock not held")
)

type RedisClient interface {
	redis.Scripter
}

type Client struct {
	client RedisClient
	tmp    []byte
	tmpMu  sync.Mutex
}

func New(client RedisClient) *Client {
	return &Client{client: client}
}

func (c *Client) Obtain(ctx context.Context, key string, ttl time.Duration, opt *Options) (*Lock, error) {
	return c.ObtainMulti(ctx, []string{key}, ttl, opt)
}

func (c *Client) ObtainMulti(ctx context.Context, keys []string, ttl time.Duration, opt *Options) (*Lock, error) {
	token := opt.getToken()
	if token == "" {
		var err error
		if token, err = c.randomToken(); err != nil {
			return nil, err
		}
	}

	value := token + opt.getMetadata()
	ttlVal := strconv.FormatInt(int64(ttl / time.Millisecond), 10)
	retry := opt.getRetryStrategy()

	if _, ok := ctx.Deadline(); !ok {
		var cancel context.CancelFunc
		ctx, cancel = context.WithDeadline(ctx, time.Now().Add(ttl))
		defer cancel()
	}

	var ticker *time.Ticker
	for {
		ok, err := c.obtain(ctx, keys, value, len(token), ttlVal)
		if err != nil {
			return nil, err
		} else if ok {
			return &Lock{
				Client: c,
				keys: keys,
				value: value,
				tokenLen: len(token),
			}, nil
		}

		backoff := retry.NextBackoff()
		if backoff < 1 {
			return nil, ErrNotObtained
		}

		if ticker == nil {
			ticker = time.NewTicker(backoff)
			defer ticker.Stop()
		} else {
			ticker.Reset(backoff)
		}

		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-ticker.C:
		}
	}
}

func (c *Client) obtain(ctx context.Context, keys []string, value string, tokenLen int, ttlVal string) (bool, error) {
	_, err := luaObtain.Run(ctx, c.client, keys, value, tokenLen, ttlVal).Result()
	if err != nil {
		if errors.Is(err, redis.Nil) {
			return false, nil
		}
		return false, err
	}
	return true, nil
}

func (c *Client) randomToken() (string, error) {
	c.tmpMu.Lock()
	defer c.tmpMu.Unlock()
	if len(c.tmp) == 0 {
		c.tmp = make([]byte, 16)
	}
	if _, err := io.ReadFull(rand.Reader, c.tmp); err != nil {
		return "", err
	}
	return base64.RawURLEncoding.EncodeToString(c.tmp), nil
}

type Lock struct {
	*Client
	keys     []string
	value    string
	tokenLen int
}

func (l *Lock) Key() string {
	return l.keys[0]
}

func (l *Lock) Keys() []string {
	return l.keys
}

func (l *Lock) Token() string {
	return l.value[:l.tokenLen]
}

func (l *Lock) Metadata() string {
	return l.value[l.tokenLen:]
}

func (l *Lock) TTL(ctx context.Context) (time.Duration, error) {
	if l == nil {
		return 0, ErrLockNotHeld
	}
	res, err := luaPTTL.Run(ctx, l.client, l.keys, l.value).Result()
	if err != nil {
		if errors.Is(err, redis.Nil) {
			return 0, nil
		}
		return 0, err
	}
	if num := res.(int64); num > 0 {
		return time.Duration(num) * time.Millisecond, nil
	}
	return 0, nil
}

func (l *Lock) Refresh(ctx context.Context, ttl time.Duration, opt *Options) error {
	if l == nil {
		return ErrNotObtained
	}
	ttlVal := strconv.FormatInt(int64(ttl / time.Millisecond), 10)
	_, err := luaRefresh.Run(ctx, l.client, l.keys, l.value, ttlVal).Result()
	if err != nil {
		if errors.Is(err, redis.Nil) {
			return ErrNotObtained
		}
		return err
	}
	return nil
}

func (l *Lock) Release(ctx context.Context) error {
	if l == nil {
		return ErrLockNotHeld
	}
	_, err := luaRelease.Run(ctx, l.client, l.keys, l.value).Result()
	if err != nil {
		if errors.Is(err, redis.Nil) {
			return ErrLockNotHeld
		}
		return err
	}
	return nil
}

type Options struct {
	Token         string
	Metadata      string
	RetryStrategy RetryStrategy
}

func (o *Options) getMetadata() string {
	if o != nil {
		return o.Metadata
	}
	return ""
}

func (o *Options) getToken() string {
	if o != nil {
		return o.Token
	}
	return ""
}

func (o *Options) getRetryStrategy() RetryStrategy {
	if o != nil && o.RetryStrategy != nil {
		return o.RetryStrategy
	}
	return NoRetry()
}

type RetryStrategy interface {
	NextBackoff() time.Duration
}

type linearBackoff time.Duration

func (r linearBackoff) NextBackoff() time.Duration {
	return time.Duration(r)
}

func LinearBackoff(backoff time.Duration) RetryStrategy {
	return linearBackoff(backoff)
}

func NoRetry() RetryStrategy {
	return linearBackoff(0)
}
