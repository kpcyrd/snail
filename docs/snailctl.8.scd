snailctl(8)

# NAME

snailctl - utility to control *snaild*(8)

# SYNOPSIS

*snailctl* [-hvV] [-S <socket>]

*snailctl decap* [-fs] [--dns <dns>] [script]

*snailctl dns* <query> [record]

*snailctl http*

*snailctl scan* <interface>

*snailctl status*

# DESCRIPTION

This program is supposed to be used with the snaild daemon.

# COMMANDS

The following commands are understood:

## decap

Manually run captive portal bypass.

## dns

Resolve a dns record using the dns recursor pushed by dhcp.

## http

Execute an http request using the dns recursor pushed by dhcp.

## connect

Create a tcp connection using the dns recursor pushed by dhcp.

## scan

Scan for wifis and display if they've been recognized by any scripts. Needs root to trigger a scan.

## status

Display the status of the current network.

# SEE ALSO

*snail*(7), *snaild*(8)
