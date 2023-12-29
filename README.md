<h1 align="center">:wrench: oreq</h1>

<h2 align="center"><em>OpenAPI Request Prompts</em></h2>
<h4 align="center">The tool for interactively creating curl arguments from OpenAPI."</h4>

[![](https://img.shields.io/github/license/uzimaru0000/oreq?style=for-the-badge)](https://github.com/uzimaru0000/tv/blob/master/LICENSE)
<!-- [![](https://img.shields.io/github/v/release/uzimaru0000/tv?style=for-the-badge)](https://github.com/uzimaru0000/tv/releases/latest) -->
<!-- ![](https://img.shields.io/github/downloads/uzimaru0000/tv/total?style=for-the-badge) -->

## WIP :construction:

### TODO

- [ ] Resolve `oneOf`, `anyOf`
- [ ] Resolve external reference

## How to use

### USAGE
```
oreq [OPTIONS] <SCHEMA>
```

### OPTIONS
```
Options:
  -b, --base-url <BASE_URL>  
  -H, --headers <HEADERS>    
  -h, --help                 Print help
  -V, --version              Print version
```

### ARGS
```
<SCHEMA>    OpenAPI schema path
```

### Example

```bash
$ oreq github.yaml
> Path /repos/{owner}/{repo}
> Method GET
> owner uzimaru0000
> repo oreq
-X GET 'https://api.github.com/repos/uzimaru0000/oreq'

$ oreq github.yaml | xargs curl
> Path /repos/{owner}/{repo}
> Method GET
> owner uzimaru0000
> repo oreq
{
  "id": 736848036,
  "node_id": "R_kgDOK-topA",
  "name": "oreq",
  "full_name": "uzimaru0000/oreq",
  "private": false,
  "owner": {
    "login": "uzimaru0000",
    "id": 13715034,
    "node_id": "MDQ6VXNlcjEzNzE1MDM0",
    "avatar_url": "https://avatars.githubusercontent.com/u/13715034?v=4",
    "gravatar_id": "",
    "url": "https://api.github.com/users/uzimaru0000",
    ....

```
