package substrate

import (
	"fmt"

	gsrpc "github.com/Snowfork/go-substrate-rpc-client"
	"github.com/Snowfork/go-substrate-rpc-client/config"
	"github.com/Snowfork/go-substrate-rpc-client/signature"
	"github.com/Snowfork/go-substrate-rpc-client/types"

	etypes "github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/types"
)

// Client struct
type Client struct {
	api      *gsrpc.SubstrateAPI
	metadata *types.Metadata
}

// NewClient foo
func NewClient() (*Client, error) {

	api, err := gsrpc.NewSubstrateAPI(config.Default().RPCURL)
	if err != nil {
		return nil, err
	}

	metadata, err := api.RPC.State.GetMetadataLatest()
	if err != nil {
		panic(err)
	}

	client := Client{
		api, metadata,
	}

	return &client, nil
}

// SubmitExtrinsic submits a packet
func (client *Client) SubmitExtrinsic(packet etypes.Packet) (bool, error) {

	from, err := signature.KeyringPairFromSecret("//Alice", "")
	if err != nil {
		return false, err
	}

	meta, err := client.api.RPC.State.GetMetadataLatest()
	if err != nil {
		return false, err
	}

	appid := types.NewBytes32(packet.AppID)
	message := types.NewBytes(packet.Message.(etypes.Unverified).Contents)

	c, err := types.NewCall(meta, "Bridge.send", appid, message)
	if err != nil {
		return false, err
	}

	ext := types.NewExtrinsic(c)

	era := types.ExtrinsicEra{IsMortalEra: false}

	genesisHash, err := client.api.RPC.Chain.GetBlockHash(0)
	if err != nil {
		panic(err)
	}

	rv, err := client.api.RPC.State.GetRuntimeVersionLatest()
	if err != nil {
		panic(err)
	}

	key, err := types.CreateStorageKey(meta, "System", "Account", from.PublicKey, nil)
	if err != nil {
		panic(err)
	}

	var accountInfo types.AccountInfo
	ok, err := client.api.RPC.State.GetStorageLatest(key, &accountInfo)
	if err != nil || !ok {
		return false, err
	}

	nonce := uint32(accountInfo.Nonce)

	o := types.SignatureOptions{
		BlockHash:   genesisHash,
		Era:         era,
		GenesisHash: genesisHash,
		Nonce:       types.NewUCompactFromUInt(uint64(nonce)),
		SpecVersion: rv.SpecVersion,
		TxVersion:   1,
		Tip:         types.NewUCompactFromUInt(0),
	}

	extI := ext

	err = extI.Sign(from, o)
	if err != nil {
		panic(err)
	}

	extEnc, err := types.EncodeToHexString(extI)
	if err != nil {
		panic(err)
	}

	fmt.Printf("Extrinsic: %#v\n", extEnc)

	_, err = client.api.RPC.Author.SubmitExtrinsic(extI)
	if err != nil {
		panic(err)
	}

	xts, err := client.api.RPC.Author.PendingExtrinsics()
	if err != nil {
		panic(err)
	}

	fmt.Printf("Pending extrinsics: %#v\n", xts)

	return true, nil
}
