# MDNS Relay

You ever wanted to split your network into multiple indepented ones, while still being able to find
your printer or join a spotify session in another network? Then this project is for you.

The purpose of this program is to relay multicast DNS (mDNS) packets between multiple private networks.
Existing solution, such a avahi allow you to do this as well but only unconditionally, relaying all questions
and answers, with this you can decide what queries get relayed and which get blocked.

## Usage
`mdns-repeater -c <config>` where config is a json file containing these options: 
```json
{
    "interfaces": "A regex used to filter interfaces on which to listen, to exclude for example the public net or VPNs completely",
    // A list of rules to be applied to incoming packages
    "rules" : [
        {
            "from": "A regex to match the incoming interface",
            "to": "The exact name of the interface to which matching packets will be relayed",
            "allow_questions": "Regex matching any questions",
            "allow_answers": "Regex matching any answers",
        }
    ]
}
```
To apply a rule has to either have a single matching question or answer.
