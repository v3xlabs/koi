{
  latest = "0.0.1";

  hashes = {
    "0.0.1" = {
      "x86_64-unknown-linux-gnu" = "sha256-aFysG5EgmVU8pxNjJoVQx1NuRj0rWRh1Pf3VD5O8tNE=";
      "aarch64-apple-darwin" = "sha256-W36Ldhh07M+oc04rY515eObt8iyH+RKgwt36GLkuvS4=";
    };
  };

  rustTargets = {
    x86_64-linux = "x86_64-unknown-linux-gnu";
    aarch64-darwin = "aarch64-apple-darwin";
  };
}
