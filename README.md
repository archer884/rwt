# Rebel Web Tokens
For a lot of reasons, I'm not a huge fan of just going into a package manager and grabbing the first library that comes up on a search relating to a task *X.* One of the big ones is that I occasionally fear what I will term "suicide by standard." Yes, I get that some group of people has gotten together and decided that *insert standard here* is a good way of doing things, but what if I don't agree?

...In this case, what I have a problem with is the idea that I should be including a header in a JWT that tells me what algorithm to use to decode it. I *know* the algorithm, guys; it's *my application* and I'm the one who *made the token.* Be reasonable.

I'm also not a huge fan of any of the existing Rust implementations of JWT in that they all seem rather opinionated as to what my payload should be. That doesn't make sense to me, either, so I didn't do that here. Actually, I don't do much at all, and that's pretty easy to see: there are only 118 lines of code in this library, and I wanna say 50 of those are dedicated to converting various other libraries' errors into my own.

tl;dr: **kitchen sink not included.**

## Features
 - Serializes and signs any payload implementing `serde::ser::Serialize`
 - Deserializes and validates any payload implementing `serde::de::Deserialize`
 - Refuses to waste bits on JWT headers
 - Gives no fucks

## Roadmap
 - Allow algorithm selection

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE][apc] or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT][mit] or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[tss]:https://theysaidso.com/api/
[hyp]:https://github.com/hyperium/hyper
[apc]:https://github.com/archer884/quoters/blob/master/LICENSE-APACHE
[mit]:https://github.com/archer884/quoters/blob/master/LICENSE-MIT
