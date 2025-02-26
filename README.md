# enka-rs

> [!WARNING]
> This wrapper is in it's early stages of development, the project lacks much polish and it is likely that many breaking changes will be pushed often.  
> The public functions in `lib::api` are the least likely to change significantly, save for the planned cache system.  
> The current focus is on fixing any present bugs, adding proper documentation, improving the type representation of the API responses and adding an optional built in cache using Redis. Support for Honkai: Star Rail and Zenless Zone Zero will only come after that.

A crate to get data from the Enka API, for Genshin Impact<!-- , Honkai: Star Rail and Zenless Zone Zero, it also includes a finder that you can use to search for names and images of game assets, for example a name or image of a character. Check [Finders](#asset-finder) for more information -->.

## Wrapper

### Getting Started
You can get the information about a player using the `gi::get_player` function. Here's an example printing it:
```rs
match gi::get_player(700935629, false, None, None).await {
    Ok(v) => println!("{v:?}"),
    Err(e) => eprintln!("{e:?}"),
};
```

<!-- ### Cache System -->
<!-- You can enable the cache system so the data gets cached until the ttl expires. Helps to prevent rate limits. -->
<!-- ```rs -->
<!-- // You can use caching by providing a client or enabling the `auto-cache` crate feature. -->
<!-- // Currently only `redis::Client` is supported. -->
<!-- let redis_client = redis::Client::open(std::env::var("REDIS_URL").unwrap_or("redis://127.0.0.1/"))?; -->
<!-- gi::get_player(700935629, false, None, None, Some(&redis_client)).await?; -->
<!-- ``` -->
<!---->
### Enka Profiles
You can get the information about the profiles, profile linked accounts and profile builds of Enka.
```rs
let username = "TheSast";
// You can reuse the same client instead of having one initialised each call by providing it to the callee.
// Currently only `reqwest::Client` is supported.
let request_client = reqwest::Client::new();

// Get and print information about someone's profile.
gi::get_profile(username, None, Some(&request_client)).await?;

// Get information about the hoyos (game accounts) of someone.
gi::get_builds(
    username,
    gi::get_hoyos(username, None, Some(&request_client))
        .await?
        .iter()
        .find(|(_, v)| matches!(v, Hoyo::Genshin(_)))
        .ok_or("No hoyos found")?
        .0,
    None,
    Some(&request_client),
)
.await?;

// Alternative method to reuse state across calls
let wrapper = Wrapper {
    user_agent: None,
    req_client: Some(request_client),
};
wrapper
    .gi()
    .get_builds(
        username,
        wrapper
            .gi()
            .get_hoyos(username)
            .await?
            .iter()
            .find(|(_, v)| matches!(v, Hoyo::Genshin(_)))
            .ok_or("No hoyos found")?
            .0,
    )
    .await?
```

## Comparasion with enkanetwork-js

| Design Choice                 | enkanetwork-js                    | enka-rs                            |
|-------------------------------|-----------------------------------|------------------------------------|
| **Language**                  | JavaScript / TypeScript           | Rust                               |<!-- | Rust (WASM support)                | -->
| **Stateful Wrapper options**  | ✅ Yes                            | ✅ Yes (`stateful` crate feature)  |
| **Flexible per-call options** | ❌ No (Options fixed on creation) | ✅ Yes                             |
| **Caching**                   | ✅ Built-in                       | ❌ Not included                    |<!-- | ✅ Optional, user-managed (Redis)  | -->

## Creator and Support

Creator: [TheSast](https://github.com/TheSast/)  
If you need support you can contact me on discord: [TheSast#5457](https://discordapp.com/TheSast#5457).  
Join the [discord server of enka](https://discord.gg/G3m7CWkssY). You can ping me there for support.  

## Credits

- [Jelosus2](https://github.com/Jelosus2)/[enkanetwork-js](https://github.com/Jelosus2/enkanetwork.js)
  - Inspiration for the `stateful` crate feature.
  - I shamelessly stole and reworded their [`README.md`](https://github.com/Jelosus2/enkanetwork.js/blob/180ccef59422f0d7adcd79ae3f2125cb880ed15e/README.md).
