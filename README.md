# Rebel Web Tokens
For a lot of reasons, I'm not a huge fan of just going into a package manager and grabbing the first library that comes up on a search relating to a task *X.* One of the big ones is that I occasionally fear what I will term "suicide by standard." Yes, I get that some group of people has gotten together and decided that *<insert standard here>* is a good way of doing things, but what if I don't agree?

...In this case, what I have a problem with is the idea that I should be including a header in a JWT that tells me what algorithm to use to decode it. I *know* the algorithm, guys; it's *my application* and I'm the one who *made the token.* Be reasonable.

I'm also not a huge fan of any of the existing Rust implementations of JWT in that they all seem rather opinionated as to what my payload should be. That doesn't make sense to me, either, so I didn't do that here. Actually, I don't do much at all, and that's pretty easy to see: there are only 118 lines of code in this library, and I wanna say 50 of those are dedicated to converting various other libraries' errors into my own.

tl;dr: **kitchen sink not included.**

## Features
 - Serializes and signs any payload implementing `serde::ser::Serialize`
 - Deserializes and validates any payload implementing `serde::de::Deserialize`
 - Refuses to waste bits on JWT headers
 - Gives no fucks 
