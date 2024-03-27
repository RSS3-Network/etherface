variable "REGISTRY" {
  default = "docker.io"
}

variable "REPOSITORY" {
  default = "rss3"
}

variable "GIT_COMMIT" {
  default = "dev"
}

variable "GIT_DATE" {
  default = "0"
}

variable "GIT_VERSION" {
  default = "docker" // original default as set in proxyd file, not used by full go stack, yet
}

variable "IMAGE_TAGS" {
  default = GIT_COMMIT // split by ","
}

variable "PLATFORMS" {
  // You can override this as "linux/amd64,linux/arm64".
  // Only a specify a single platform when `--load` ing into docker.
  // Multi-platform is supported when outputting to disk or pushing to a registry.
  // Multi-platform builds can be tested locally with:  --set="*.output=type=image,push=false"
  default = "linux/amd64"
}

group "default" {
  targets = [
    "etherface",
    "etherface-ui"
  ]
}

target "etherface-ui" {
  # dockerfile = "etherface-ui/Dockerfile"
  context = "etherface-ui/"
  args = {
    GIT_COMMIT = "${GIT_COMMIT}"
    GIT_DATE   = "${GIT_DATE}"
  }
  platforms = split(",", PLATFORMS)
  tags      = [for tag in split(",", IMAGE_TAGS) : "${REGISTRY}/${REPOSITORY}/etherface-ui:${tag}"]

}

target "etherface" {
  context    = "."
  args = {
    GIT_COMMIT = "${GIT_COMMIT}"
    GIT_DATE   = "${GIT_DATE}"
  }
  platforms = split(",", PLATFORMS)
  tags      = [for tag in split(",", IMAGE_TAGS) : "${REGISTRY}/${REPOSITORY}/etherface:${tag}"]
}

target "etherface-rest" {
  dockerfile = "etherface-rest/Dockerfile"
  context    = "."
  args = {
    GIT_COMMIT = "${GIT_COMMIT}"
    GIT_DATE   = "${GIT_DATE}"
  }
  platforms = split(",", PLATFORMS)
  tags      = [for tag in split(",", IMAGE_TAGS) : "${REGISTRY}/${REPOSITORY}/etherface-rest:${tag}"]
}
