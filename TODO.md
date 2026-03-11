* dynamic generated types for lua libraries
* Bindgen generator
* Make the polyfill better

* update docs for the new breaking changes. mainly on assets
* use uber apk signer
* build wasm and make it a target for kaledis
* make assets behave more like the typescript alias
* make the hot module replacement better
* generate updated types for love2d 12 

https://love2d.org/wiki/12.0
Missing apis :
* Input, Math, Physics, Window, Graphics

Added:
* General(no lua-https yet), Data, Filesystem, Audio

Optional:
* be used in pesde like dalbit
* Add modding support
* vite inspired plugin system
* Live variables ( this will now be a plugin )