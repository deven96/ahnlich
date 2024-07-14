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


type Version struct {
	Major uint8
	Minor uint16
	Patch uint16
}

type Versions struct {
	Protocol Version
	Client Version
}

func getVersion( content []byte, regexFormat string) (Version, error) {
	// Use regex to find the version
	re := regexp.MustCompile(regexFormat)
	match := re.FindStringSubmatch(string(content))
	if match == nil {
		return Version{}, &utils.AhnlichClientException{Message: "Unable to Parse Protocol Version"}
	}
	// Split the version string and convert to integers
	strVersion := match[1]
	versionParts := strings.Split(strVersion, ".")
	if len(versionParts) != 3 {
		return Version{}, &utils.AhnlichClientException{Message: "Invalid version format"}
	}
	major, err := strconv.Atoi(versionParts[0])
	if err != nil {
		return Version{}, &utils.AhnlichClientException{Message: "Invalid major version number"}
	}
	minor, err := strconv.Atoi(versionParts[1])
	if err != nil {
		return Version{}, &utils.AhnlichClientException{Message: "Invalid minor version number"}
	}
	patch, err := strconv.Atoi(versionParts[2])
	if err != nil {
		return Version{}, &utils.AhnlichClientException{Message: "Invalid patch version number"}
	}
	return Version{Major: uint8(major), Minor: uint16(minor), Patch: uint16(patch)}, nil
}

func GetVersions() (Versions, error) {
	var content []byte
	// Read the VERSION file
	if versionContent == "" {
		content, err := os.ReadFile(BaseDir+ VersionFile)
		if err != nil {
			return Versions{}, &utils.AhnlichClientException{Message: "Unable to read VERSION file"}
			}
	} else {
		content = []byte(versionContent)
	}
	// `CLIENT="([^"]+)"`
	// `PROTOCOL="([^"]+)"`
	protocolVersion, err := getVersion(content,`PROTOCOL="([^"]+)"`)
	if err != nil {
		return Versions{}, err
	}
	clientVersion, err := getVersion(content,`CLIENT="([^"]+)"`)
	if err != nil {
		return Versions{}, err
	}
	return Versions{Protocol: protocolVersion, Client: clientVersion}, nil
}