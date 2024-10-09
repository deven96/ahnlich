package ahnlichclientgo

import (
	"os"
	"regexp"
	"strconv"
	"strings"

	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"

	_ "embed"
)

//go:embed VERSION
var versionContent string // VERSION file content is embedded in the binary

type version struct {
	Major uint8
	Minor uint16
	Patch uint16
}

// AhnlichVersion contains the protocol and client versions
type AhnlichVersion struct {
	Protocol version
	Client   version
}

func getVersion(content []byte, regexFormat string) (version, error) {
	// Use regex to find the version
	re := regexp.MustCompile(regexFormat)
	match := re.FindStringSubmatch(string(content))
	if match == nil {
		return version{}, &utils.AhnlichClientException{Message: "Unable to Parse Protocol version"}
	}
	// Split the version string and convert to integers
	strVersion := match[1]
	versionParts := strings.Split(strVersion, ".")
	if len(versionParts) != 3 {
		return version{}, &utils.AhnlichClientException{Message: "Invalid version format"}
	}
	major, err := strconv.Atoi(versionParts[0])
	if err != nil {
		return version{}, &utils.AhnlichClientException{Message: "Invalid major version number"}
	}
	minor, err := strconv.Atoi(versionParts[1])
	if err != nil {
		return version{}, &utils.AhnlichClientException{Message: "Invalid minor version number"}
	}
	patch, err := strconv.Atoi(versionParts[2])
	if err != nil {
		return version{}, &utils.AhnlichClientException{Message: "Invalid patch version number"}
	}
	return version{Major: uint8(major), Minor: uint16(minor), Patch: uint16(patch)}, nil
}

// GetVersions returns the protocol and client versions
func GetVersions() (AhnlichVersion, error) {
	var content []byte
	var err error
	// Read the VERSION file if versionContent is empty
	if versionContent == "" {
		baseDir, err := utils.GetProjectRoot()
		if err != nil {
			return AhnlichVersion{}, &utils.AhnlichClientException{Message: "Unable to get project root"}
		}
		content, err = os.ReadFile(baseDir + VersionFile)
		if err != nil {
			return AhnlichVersion{}, &utils.AhnlichClientException{Message: "Unable to read VERSION file"}
		}
	} else {
		content = []byte(versionContent)
	}

	// `CLIENT="([^"]+)"`
	// `PROTOCOL="([^"]+)"`
	protocolVersion, err := getVersion(content, `PROTOCOL="([^"]+)"`)
	if err != nil {
		return AhnlichVersion{}, err
	}
	clientVersion, err := getVersion(content, `CLIENT="([^"]+)"`)
	if err != nil {
		return AhnlichVersion{}, err
	}

	return AhnlichVersion{Protocol: protocolVersion, Client: clientVersion}, nil
}
