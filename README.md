# Cat | bash CLI

A flexible command-line utility for piping content from files into a `bash` shell and capturing/modifying output in various ways. This tool is designed for scripting and automating command execution workflows with optional post-processing.

---

## Features

- Pipe file contents into `bash` (`cat file | bash`)
- Write or capture outputs from commands
- Modify captured output with optional arguments
- Support for inline input, output redirection, and temporary files
- Error propagation and validation for CLI misuse

---

## Installation
Project dependancies come in a direnv + shell.nix combination. You may handle them as you prefer.

Clone and build using `cargo`:

```bash
git clone https://github.com/Auxidity/catbash.git
cd catbash
cargo build --release
```

## Usage
catbash [FLAGS] [OPTIONS] [FILE]

## CLI Flags and Options
Flag	                            Description
-i, --input <string>	            Input string to write to a file and catbash. Accepts "quoted args" or simple commands
-o, --output <file>	                File to write input to and then execute via cat file
-c, --capture	                    Capture the stdout of cat file
-t, --target <file>	                Write the captured (or modified) output to a file
-a, --arguments <string>            Arguments used to post-process captured output (e.g., pipe to grep)
-f, --arguments_from_file <file>	Same as -a but reads the arguments from a file
[FILE]	A single file to directly catbash when no flags are present

## Examples
Simple useage:
catbash my_script.sh 

Storing a simple command for reuse to output
catbash -i "ls -l" -o out.sh 

Capturing and modifying the output of out.sh with arguments and displaying the result in stdout
catbash -i "ls -l" -o out.sh -c -a "| grep txt"

Capturing and modifying the output of out.sh with arguments and then storing the result to a file
catbash -i "ls -l" -o out.sh -c -t out2.sh -a "| grep txt"

Something more complex :

The following in a file called test_post.txt

```
| echo "curl -X POST http://localhost:6969/api/blogs -H \"Content-Type: application/json\" -H \"Authorization: Bearer $(cat)\" -d '{\"title\": \"Sample Blog Title\", \"author\": \"Sample Author\", \"url\": \"http://sampleurl.com\"}'"
```

```bash
catbash -i 'echo "2"' -o test.txt -c -f test_post.txt 
```

returns the following: 

```

curl -X POST http://localhost:6969/api/blogs -H "Content-Type: application/json" -H "Authorization: Bearer 2" -d '{"title": "Sample Blog Title", "author": "Sample Author", "url": "http://sampleurl.com"}'
```

Note: to achieve the same thing with -a flag, the actual string is a little different compared to the file stored one. The escaped characters would look to the program like multiple files inputs, which it doesn't accept.

In order to achieve the same thing with -a flag, you would have to run

```bash
catbash -i 'echo "2"' -o test.txt -c -a '| echo "curl -X POST http://localhost:6969/api/blogs -H "Content-Type: application/json" -H "Authorization: Bearer $(cat)" -d '"'"'{ "title": "Sample Blog Title", "author": "Sample Author", "url": "http://sampleurl.com" }'"'"'"'
```



## Constraints & Validation

The --capture flag requires --output

--target, --arguments, or --arguments-from-file require both --output and --capture

Cannot use both --arguments and --arguments-from-file

Cannot mix positional file with flags

Cannot accept multiple files when catbashing (e.g. catbash script1.sh script2.sh)

Note: Echoing raw shell commands (like echo "ls -l") may behave unexpectedly due to shell quoting/escaping. Handling nested strings is not perfect.
