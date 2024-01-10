## How to deploy our developer site(dev-site)

### 1. Checkout to gh-pages which is the branch of dev-site
```sh
$ cd $(islet-project/islet)
$ git checkout gh-pages
```

### 2. Rebase to latest main branch
```sh
$ git rebase main
```

### 3. Generate dev-site docs
```sh
$ ./scripts/make_doc.sh
```

### 4. Commit the result docs
```sh
$ git add docs/*
$ git commit -s -m "Bump to latest main branch"
```

### 5. Deploy the docs to the remote site
```sh
$ git push origin gh-pages --force
```

That's it! You can check the status of deploy [here](https://github.com/islet-project/islet/actions).
