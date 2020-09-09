package ethereum

import (
	"context"

	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/secp256k1"
)

// Connection ...
type Connection struct {
	endpoint string
	kp       *secp256k1.Keypair
	client   *ethclient.Client
	log      *logrus.Entry
}

// NewConnection ...
func NewConnection(endpoint string, kp *secp256k1.Keypair, log *logrus.Entry) *Connection {
	return &Connection{
		endpoint: endpoint,
		kp:       kp,
		log:      log,
	}
}

func (co *Connection) Connect(ctx context.Context) error {

	client, err := ethclient.Dial(co.endpoint)
	if err != nil {
		return err
	}

	chainID, err := client.NetworkID(ctx)
	if err != nil {
		return err
	}

	co.log.WithFields(logrus.Fields{
		"endpoint": co.endpoint,
		"chainID":  chainID,
	}).Info("Connected to chain")

	co.client = client

	return nil
}

// Close terminates the client connection
func (co *Connection) Close() {
	if co.client != nil {
		co.client.Close()
	}
}