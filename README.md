# catbash
Cat | Bash

## Usage
If you have a file that has the following typed inside it: echo "Hello, World!" 
calling catbash hello_world.txt will result in 'catbashing' the content, resulting in cat echo "Hello, World!" | bash to be given to execute to the shell.

Additionally, you can optionally specify arguments for more options.

You can do something like

catbash -i "ls"  -o out.txt -c -t test.txt -a "| grep Cargo"

where we declare to use "ls" as the command, write it to out.txt for further use, -c to signify that the output of catbash should be captured, -t to store the output of captured output in test.txt, and -a is a custom argument which gets applied to the captured output, before it gets stored.

So in this case, if this git is used as the place where you execute the above command, it would first list all the files, store the command 'ls' to out.txt, then grep for Cargo on the result of invoking ls, which would result in Cargo.lock\nCargo.toml\n . This is then stored to test.txt .

## Why?
I have found to be typing cat somefile.txt | bash repeatedly over the years to automate scripts that I want to run often on demand, but not on a cronjob level.

For example, I might have a c program that I want to compile with loads of flags, and vast majority of the time its with the same flags. I like to write the entire thing into a text file and just cat | bash it.

Another example is when I need to use curl for a webserver that I am debugging. I prefer to do my debugging from the CLI and not use fancy tools, so the ability to test endpoints from CLI the exact way I want to tends to be some form of curl -X POST localhost:6969/api/give_me_auth_key , and then wanting to store that auth key to do something like

curl -X POST http://localhost:6969/api/blogs \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VybmFtZSI6InJvb3QiLCJpZCI6IjY3YjVmMTRlMjEyNDhjYjExMDQwNmMzMSIsImlhdCI6MTczOTk3NzA5MiwiZXhwIjoxNzM5OTgwNjkyfQ.UZxwxdExDRbzrCZbs0B79eHEljNtUUdFl-RqagT_sQM" \
  -d '{
    "title": "Sample Blog Title",
    "author": "Sample Author",
    "url": "http://sampleurl.com"
  }'

and automatically replace the Bearer field with whatever the first POST responds with.

The solution leaves all responsibility for you to figure out how to handle the string processing, catbash just gives you the string as is (or stored to a file), and if you can figure out how to string process it from echo "{my output}", it allows you to do so. Alternatively you could manually modify the stored file, and then catbash that.

For example to modify the Authorization bearer field on the captured response, you can type this monstrosity

catbash -i 'echo "2"' -o test.txt -c -a '| echo "curl -X POST http://localhost:6969/api/blogs   -H \"Content-Type: application/json\"   -H \"Authorization: Bearer $(cat)\"   -d '"'"'{ "title": "Sample Blog Title", "author": "Sample Author", "url": "http://sampleurl.com" }'"'"'"'

and it would replace the output to :

curl -X POST http://localhost:6969/api/blogs   -H "Content-Type: application/json"   -H "Authorization: Bearer 2"   -d '{ title: Sample Blog Title, author: Sample Author, url: http://sampleurl.com }'

In future, Im looking to make it so that any string manipulation you'd want to do can also be taken from a file and used as is, because having to type all that into terminal more than once to achieve that is a bit silly.
