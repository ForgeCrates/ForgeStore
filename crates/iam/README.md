will store keys in redis at each rqeuest for fast authentication (replace oldest used)

will store roles,policies and users data in metadata

roles is collection of policies
can give custom role by manually controlling policies

sts/ Issue, validate, and revoke temporary security credentials and session tokens.


use access key and secret key with HMAC for authentication
add roles and policies for authorization


metadata nodes will provide db url and operations will be written here