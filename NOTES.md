- "comma separated list" should somehow be part of the type
  - technically, there are a few ways to infer this without making the change:
    - if the default value is comma separated
    - if the description includes the token `comma-separated`
    - 
- any `available` should be `enum[string]`
- `bucketPermissions` is inconsistent in `Body`
  - if it's empty, it's an array. otherwise, it's an object. i understand that this is quirk of PHP, but how should we handle it?
    - we could have configurable overrides, e.g. key 'bucketPermissions' is always an object.
- why do `parameters` sometimes include attributes? e.g. `async` in `Bucket Sharing`
- some variants are only listed as text, instead of being encoded as separate requests with different responses, or something similar.
  - example is `Component List`, which has the `exclude` parameter (should be an attribute), and the response depends on this parameter, but there is no example of it.
- `async` means there are two different responses, but again, it is not encoded in any way

api blueprint is not expressive enough to be transformed into TypeScript