[![progress-banner](https://backend.codecrafters.io/progress/http-server/0ffcc459-852d-460b-a220-40a4309759a0)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own HTTP server" Challenge](https://app.codecrafters.io/courses/http-server/overview).

[HTTP](https://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol) is the
protocol that powers the web. In this challenge, you'll build a HTTP/1.1 server
that is capable of serving multiple clients.

Along the way you'll learn about TCP servers,
[HTTP request syntax](https://www.w3.org/Protocols/rfc2616/rfc2616-sec5.html),
and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Passing the first stage

The entry point for your HTTP server implementation is in `src/main.rs`. Study
and uncomment the relevant code, and push your changes to pass the first stage:

```sh
git commit -am "pass 1st stage" # any msg
git push origin master
```

Time to move on to the next stage!

# Stage 2 & beyond

Note: This section is for stages 2 and beyond.

1. Ensure you have `cargo (1.82)` installed locally
1. Run `./your_program.sh` to run your program, which is implemented in
   `src/main.rs`. This command compiles your Rust project, so it might be slow
   the first time you run it. Subsequent runs will be fast.
1. Commit your changes and run `git push origin master` to submit your solution
   to CodeCrafters. Test output will be streamed to your terminal.

# TODO

This is a collection of TODOs of possible improvements/refactors that I feel would make this
project more elegant / reusable / etc, but do not have the time for right now:
- [ ] Create a `Server` struct that keeps configuration, eg, `--directory`
- [ ] Use of macros for building routes
- [ ] Builder pattern
- [ ] Separate framework from functionality for passing CodeCrafters test(s)
- [ ] Due to adding support for reading body, `Request::decode` got a little unwieldy. I am tempted to have `new()` read all the bytes from the stream into `bytes_received`, so `Request` can use references.
- [ ] Validate that the body sent is the `content-length` client provided
- [ ] 100% branch coverage (time consuming 😅)
- [ ] Various improvements to testing - should the CodeCrafters tests be integration? should there be helpers for parsing responses? etc...
- [ ] Profile performance and fuzz
