# patisserie

`patisserie` is a command line interface for [Pastery][pastery], the sweetest
pastebin in the world.

## Usage

```
Usage: patisserie [OPTIONS] [PATH]

Arguments:
  [PATH]
          The path of the file to upload.

          If not provided, the file will be read from standard input.

Options:
      --api-key <API_KEY>
          Your pastery API key.

          If not provided, it will be read from the PASTERY_API_KEY environment
          variable.

          You can find this at https://www.pastery.net/account/.

  -h, --help
          Print help (see a summary with '-h')
```

[pastery]: https://www.pastery.net
