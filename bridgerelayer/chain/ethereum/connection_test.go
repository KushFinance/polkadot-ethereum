// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum_test

import (
	"context"
	"testing"

	"github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/crypto/secp256k1"
)

func TestConnect(t *testing.T) {
	log := logrus.NewEntry(logrus.New())

	conn := ethereum.NewConnection("ws://localhost:9545",  secp256k1.Alice(), log)
	err := conn.Connect(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	conn.Close()
}
