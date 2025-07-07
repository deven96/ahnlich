package ahnlichgotest

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestTestAreRunning(t *testing.T) {
	t.Run("TestTestAreRunning", func(t *testing.T) {
		require.True(t, true)
	})
}
