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

  -d, --duration <DURATION>
          The duration that this paste will live for.

          After this time, the paste will be deleted.

          You can specify a period of minutes or a value followed by one of the
          following units: m(inute), h(our), d(ay), mo(nth), y(ear)

          [default: 1d]

  -l, --lang <LANGUAGE>
          The language for the paste.

          If not provided, patisserie will attempt to guess based on the file
          extension. You can use the special value "autodetect" to have pastery
          detect the language.

  -t, --title <TITLE>
          The title of the paste.

          If not provided, the name of the file will be used instead.

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
