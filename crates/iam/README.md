will store keys in redis at each rqeuest for fast aithentication
will store roles,policies and users data in metadata

roles is collection of policies
can give custom role by manually controlling policies

sts/ Issue, validate, and revoke temporary security credentials and session tokens.


at gateway request would be canonicalized->can req hashed with sha256