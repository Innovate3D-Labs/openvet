# Security

This repository is a draft specification and a library. It has no deployed service.

If you find a problem in the state machine or the validation rules that would let a peer skip an authorisation check, double-sell a pool, or treat an operational meter as fiscal, open an issue or send a patch. There is no security team and no bounty.

Do not send vulnerability reports that include live credentials, vehicle identifiers, or customer data. The examples in this repo are fictional. Keep them that way.

A peer authenticated with mTLS is trusted for the party id in its certificate. The 0.1 draft does not define how that binding is written into the certificate, and it does not define revocation. Both are open. Until they are written down, any deployment is inventing a trust model the spec will not back.
