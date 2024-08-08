package utils

// Custom error type
type AhnlichClientException struct {
	Message string
}

func (e *AhnlichClientException) Error() string {
	return e.Message
}
