# Compile veraison from the CCA demo tutorial

[Link to CCA Demo steps from veraison docs](https://github.com/veraison/docs/blob/main/demo/cca/manual-end-to-end.md)

## Preparing Jsons :)

### Create an config file for rocli
```yml
lang: "en-US"

tag_identity:
  id: "366D0A0A-5988-45ED-8488-2F2A544F6242"
  version: 0

entities:
  - name: "ACME Ltd."
    regid: "https://example.com"
    comid_roles:
      - tagCreator
      - creator
      - maintainer
    corim_roles:
      - manifestCreator


validity:
  not-before: "2021-12-31T00:00:00Z"
  not-after: "2025-12-31T00:00:00Z"

profiles:
  - "http://arm.com/cca/ssd/1"
  - "http://arm.com/CCA-SSD/1.0.0"

environment:
  vendor: "ACME"
  model: "ACME"
```

### Create `endorsements.json`

```sh
rocli --config demo/config.yml -o endorsements.json \
    --token demo/token/token.bin endorsements \
    --cpak demo/claims/cpak_public.pem
```

### Create `refvals.json`

```sh
rocli --config demo/config.yml -o refvals.json \
    --token demo/token/token.bin refvals
```

### Create `corim.json`

```sh
rocli --config demo/config.yml -o corim.json \
    --token demo/token/token.bin corim
```
