snaild(8)

# NAME

snaild - parasitic network manager

# SYNOPSIS

*snaild* [-hvV] [-S <socket>] <interface>

# DESCRIPTION

The snaild process takes an interface and manages a dhcpcd sub process. The hooks that are triggered by *dhcpcd* are reported back to snaild and the data is available to *snailctl*.

It also tests for working network connectivity and tries to executed scripts with the settings pushed by dhcp to enable a working network connection if possible, eg by submitting captive portal forms. Those scripts are provided by the user, see *snail*(7).

# SEE ALSO

*snail*(7), *snailctl*(8), *dhcpcd*(8)
