# Release Notes

Release notes for Trippy 0.6.0 onwards. See also the [CHANGELOG](CHANGELOG.md).

# 0.10.0

## Highlights

The first release of 2024 is packed with new features, such as customizable columns, jitter calculations, Dublin tracing strategy for IPv6/UDP, support for IPinfo GeoIp files, enhanced DNS resolution with IPv6/IPv4 fallback and CSS named colors for the TUI as well as a number of bug fixes.  Since the last release there has also been a significant improvement in automated testing, notably the introduction of TUN based simulation testing for IPv4.

### Customize Columns

#### Customize Columns in TUI

It is now possible to customize which columns are shown in the TUI and to adjust the order in which they are displayed.  This customization can be made from within the TUI or via configuration.

To customize the columns from the TUI you must open the settings dialog (`s` key) and navigating to the new `Columns` tab (left and right arrow keys).  From this tab you can select the desired column (up and down arrow keys) and toggle the column visibility on and off (`c` key) or move it left (`,` key) or right (`.` key) in the list of columns.

<img width="60%" alt="columns" src="https://github.com/fujiapple852/trippy/blob/master/assets/0.10.0/columns_settings.png">

You can supply the full list of columns, in the desired order, using the new `--tui-custom-columns` command line argument.  The following example specifies the standard list of columns in the default order:

```shell
trip example.com --tui-custom-columns holsravbwdt
```

Alternatively, to make the changes permanent you may add the `tui-custom-columns` entry to the `tui` section of the Trippy configuration file:

```toml
[tui]
tui-custom-columns = "holsravbwdt"
```

Note that the value of `tui-custom-columns` can be seen in the corresponding field of the `Tui` tab of the settings dialog and will reflect any changes made to the column order and visibility via the Tui.  This can be useful as you may copy this value and use it in the configuration file directly.

<img width="60%" alt="tui-custom-columns" src="https://github.com/fujiapple852/trippy/blob/master/assets/0.10.0/tui_settings.png">

#### New Columns

This release also introduced several new columns, all of which are hidden by default.  These are:

- Last source port: The source port for last probe for the hop
- Last destination port: The destination port for last probe for the hop
- Last sequence number: The sequence number for the last probe for the hop
- Jitter columns: see the "Calculate and Display Jitter" section below

See the [Column Reference](https://github.com/fujiapple852/trippy#column-reference) for a full list of all available columns.

#### Column Layout Improvement

The column layout algorithm used in the hop table has been improved to allow the maximum possible space for the `Host` column.  The width of the `Host` column is now calculated dynamically based on the terminal width and the set of columns currently configured.

### Calculate and Display Jitter

Trippy can now calculate and display a variety of measurements related to _jitter_ for each hop.  Jitter is a measurement of the difference in round trip time between consecutive probes. Specifically, the following new calculated values are available in Trippy `0.10.0`:

- Jitter: The round-trip-time (RTT) difference between consecutive rounds for the hop
- Average Jitter: The average jitter of all probes for the hop
- Maximum Jitter: The maximum jitter of all probes for the hop
- Inter-arrival Jitter: The smoothed jitter value of all probes for the hop

These values are always calculated and are included in the `json` report.  These may also be displayed as columns in the TUI, however they are not shown by default.  To enabled these columns in the TUI, please see the [Column Reference](https://github.com/fujiapple852/trippy#column-reference).

<img width="60%" alt="jitter" src="https://github.com/fujiapple852/trippy/blob/master/assets/0.10.0/jitter_columns.png">

### Dublin Tracing Strategy for IPv6/UDP

The addition of support for the [dublin](https://github.com/insomniacslk/dublin-traceroute) tracing strategy for IPv6/UDP marks the completion of a multi-release journey to provide support for both Dublin and [paris](https://github.com/libparistraceroute/libparistraceroute/wiki/Checksum) tracing strategies for both IPv4/UDP and IPv6/UDP.

As a reminder, unlike classic traceroute and MTR, these alternative tracing strategies do not encode the probe sequence number in either the src or dest port of the UDP packet, but instead use other protocol and address family specific techniques.  Specifically, the Dublin tracing strategy for IPv6/UDP varies the length of the UDP payload for this purpose.

By doing so, these strategies are able to keep the src and dest ports fixed which makes it much more likely (though not guaranteed) that each round of tracing will follow the same path through the network (note that this is not true for the return path).

The following command runs an IPv6/UDP trace using the Dublin tracing strategy with fixed src and dest ports:

```shell
trip example.com --udp -6 -R dublin -S 5000 -P 3500
```

Note that, for both Paris and Dublin tracing strategies, if you fix either the src or dest ports (but _not_ both) then Trippy will vary the unfixed port _per round_ rather than _per hop_.  This has the effect that all probes _within_ a round will likely follow the same network path but probes _between_ round will follow different paths.  This can be useful in conjunction with flows (`f` key) to visualize the various paths packet flow through the network.  See this [issue](https://github.com/fujiapple852/trippy/issues/1007) for more details.

<img width="60%" alt="ipv6_dublin" src="https://github.com/fujiapple852/trippy/blob/master/assets/0.10.0/dublin_ipv6_src_dest_seq_columns.png">

With UDP support for the Paris and Dublin tracing strategies now complete, what remains is adding support for these for the TCP protocol.  Refer to the [ECMP tracking issue](https://github.com/fujiapple852/trippy/issues/274) for details.

### IPinfo GeoIp Provider

Trippy currently supports the ability to lookup and display GeoIp information from MMDB files, but prior to `0.10.0` only the [MaxMind](https://www.maxmind.com) "GeoLite2 City" (and lite) MMDB files were supported.  This release introduces support for the "IP to Country + ASN Database" and "IP to Geolocation Extended Database" MMDB files from [IPinfo](https://ipinfo.io).

The "IP to Country + ASN Database" MMDB file provided by IPinfo can be used as follows:

```shell
trip example.com --geoip-mmdb-file /path/to/country_asn.mmdb --tui-geoip-mode short
```

These settings can be made permanent by setting the following values in the `tui` section of the configuration file:

```toml
[tui]
geoip-mmdb-file = "/path/to/country_asn.mmdb"
tui-geoip-mode = "short"
```

### Enhanced DNS Resolution with IPv4/IPv6 Fallback

When provided with a DNS name such as `example.com` Trippy tries to resolve it to an IPv4 or an IPv6 address and fails if no such IP exists for the configured `addr-family` mode, which must be either IPv4 or IPv6.

Starting from version `0.10.0`, Trippy can be configured to support `ipv4-then-ipv6` and `ipv6-then-ipv4` modes for `addr-family`.  In the new `ipv4-then-ipv6` mode Trippy will first attempt to resolve the given hostname to an IPv4 address and, if no such address exists, it will attempt to resolve to an IPv6 address and only fail if neither are available (and the opposite for the new `ipv6-then-ipv4` mode).  The `addr-family` mode may also be set to be `ipv4` or `ipv6` for IPv4 only and IPv6 only respectively.

To set the `addr-family` to be IPv6 with fallback to IPv4 you can set the `--addr-family` command line parameter:

```shell
trip example.com --addr-family ipv6-then-ipv4
```

To make the change permanent you can set the `addr-family` value in the `strategy` section of the configuration file:

```toml
[strategy]
addr-family = "ipv6-then-ipv4"
```

Note that Trippy supports both the `addr-family` entry in the configuration file and also the `--ipv4` (`-4`) and `--ipv6` (`-6`) command line flags, all of which are optional. The command line flags (which are mutually exclusive) take precedence over the config file entry and if neither are provided there it defaults to `ipv4-then-ipv6`.

### Extended Colors in TUI

Trippy allows the theme to be customized and supports the named [ANSI colors](https://en.wikipedia.org/wiki/ANSI_escape_code#Colors):

Black, Red, Green, Yellow, Blue, Magenta, Cyan, Gray, DarkGray, LightRed, LightGreen, LightYellow, LightBlue, LightMagenta, LightCyan, White

The `0.10.0` release adds support for CSS [named colors](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) (e.g. SkyBlue).  Note that these are only supported on some platforms and terminals and may not render correctly elsewhere.

See the [Theme Reference](https://github.com/fujiapple852/trippy#theme-reference)

### Simulation Testing

Manually testing all Trippy features in all modes and on all supported platforms is an increasingly time consuming and error prone activity.  Since the last release a significant effort has been made to increase the testing coverage, including unit and integration testing.

In particular, the introduction of simulation testing allows for full end-to-end testing of all modes and features on Linux, macOS and Windows without the need to mock or stub any behaviour _within_ Trippy.

This is achieved by creating a [TUN](https://en.wikipedia.org/wiki/TUN/TAP) device to simulate the behavior of network nodes, responding to various pre-configured scenarios like packet loss and out-of-order arrivals.

Whilst not a change that directly benefits end users, this new testing approach should reduce the effort needed to test each release of Trippy and help improve the overall reliability of the tool.

Note that the simulation testing is currently only supported for IPv4.  See the [Integration Testing](https://github.com/fujiapple852/trippy/issues/759) tracking issue for more details.

### Thanks

My thanks to all Trippy contributors, package maintainers and community members.

Feel free to drop by the Trippy Matrix room for a chat:

[![#trippy-dev:matrix.org](https://img.shields.io/badge/matrix/trippy-dev:matrix.org-blue)](https://matrix.to/#/#trippy-dev:matrix.org)

Happy Tracing!

# 0.9.0

## Highlights

Trippy `0.9.0` introduces many new features, including tracing flows and ICMP extensions, the expansion of support for the Paris tracing strategy to encompass IPv6/UDP, an unprivileged execution mode for macOS, a hop privacy mode and many more.  Additionally, this release includes several important bug fixes along with a range of new distribution packages.

### Tracing Flows

#### Flow Id

A tracing flow represents the sequence of hosts traversed from the source to the target.  Trippy is now able to identify individual flows within a trace and assign each a unique flow id.  Trippy calculate a flow id for each round of tracing, based on the sequence of hosts which responded during that round, taking care to account for rounds in which only a subset of hosts responded.  Tracing statistics, such as packet loss % and average RTT are recorded on a per-flow basis as well as being aggregated across all flow.

Tracing flows adds to the existing capabilities provided by Trippy to assist with [ECMP](https://en.wikipedia.org/wiki/Equal-cost_multi-path_routing) (Equal-Cost Multi-Path Routing) when tracing with UDP and TCP protocols. Some of these capabilities, such as the [paris](https://github.com/libparistraceroute/libparistraceroute/wiki/Checksum) and [dublin](https://github.com/insomniacslk/dublin-traceroute) tracing strategies, are designed to _restrict_ tracing to a single flow, whilst others, such as the hop detail navigation mode (introduce in the last release) and tracing flows, are designed to help _visualize_ tracing data in the presence of multiple flows. See the `0.8.0` [release note](https://github.com/fujiapple852/trippy/releases/tag/0.8.0) for other such capabilities.

#### Tracing Flows in the TUI

The TUI has been enhanced with a new mode to help visualise flows. This can be toggled on and off with the `toggle-flows` command (bound to the `f` key by default).

When toggled on, this mode display flow information as a chart in a new panel above the hops table.  Flows can be selected by using the left and right arrow keys (default key bindings).  Flows are sorted by the number of rounds in which a given flow id was observed, with the most frequent flow ids shown on the left.  When entering this mode flow id 1 is selected automatically. The selected flow acts as a filter for the other parts of the TUI, including the hops table, chart and maps views which only show data relevant to that specific flow.

<img width="60%" alt="flows" src="https://github.com/fujiapple852/trippy/blob/master/assets/0.9.0/flows.png">

When toggled off, Trippy behaves as it did in previous versions where aggregated statistics (across all flows) are shown.  Note that per-flow data is always recorded, the toggle only influences how the data is displayed.

The number of flows visible in the TUI is limited and can be controlled by the `tui-max-flows` configuration items which can be set via the command line or via the configuration file.  By default up to 64 flows are shown.

The flows panel, as with all other parts of the TUI, can also be themed, see the [theme reference](https://github.com/fujiapple852/trippy#theme-reference) for details.

#### Flow Reports

As well as visualising flows in the TUI, Trippy `0.9.0` introduces two new reports which make use of the tracing flow data.

The new `flows` report mode records and print all flows observed during tracing.

The following command will run a TCP trace for 10 round and report all of the flows observed:

```shell
trip example.com --tcp -m flows -C 10
```

Sample output (truncated) showing three unique flows:

```text
flow 1: 192.168.1.1, 10.193.232.245, 218.102.40.38, 10.195.41.9, 172.217.27.14
flow 2: 192.168.1.1, 10.193.232.245, 218.102.40.22, 10.195.41.17, 172.217.27.14
flow 3: 192.168.1.1, 10.193.232.245, 218.102.40.38, 10.195.41.1, 172.217.27.14
```

Another new report, `dot`, outputs a [GraphViz](https://graphviz.org/) [`DOT`](https://graphviz.org/doc/info/lang.html) format chart of all hosts observed during tracing.

The following command will run a TCP trace for 10 round and output a graph of flows in `DOT` format:

```shell
trip example.com --tcp -m dot -C 10
```

If you have a tool such as `dot` (Graphviz) installed you can use this to rendered the output in various formats, such as PNG:

```shell
trip example.com --tcp -m dot -C 10 | dot -Tpng > path.png
```

Sample output:

<img width="60%" alt="dot" src="https://github.com/fujiapple852/trippy/blob/master/assets/0.9.0/dot.png">

### ICMP Extensions

#### Parsing Extensions

Trippy `0.9.0` adds the ability to parse and display ICMP Multi-Part Messages (aka extensions).  It supports both compliant and non-compliant ICMP extensions as defined in [section 5 of rfc4884](https://www.rfc-editor.org/rfc/rfc4884#section-5).

Trippy is able to parse and render any generic Extension Object but is also able to parse some well known Object Classes, notably the MPLS class.

Support for [additional classes](https://www.iana.org/assignments/icmp-parameters/icmp-parameters.xml#icmp-parameters-ext-classes) will be added to future versions of Trippy, see the ICMP Extensions [tracking issue](https://github.com/fujiapple852/trippy/issues/33).

Parsing of ICMP extensions can be enabled by setting the `--icmp-extensions` (`-e`) command line flag or by adding the `icmp-extensions` entry in the `strategy` section of the configuration file:

```toml
[strategy]
icmp-extensions = true
```

#### ICMP Extensions in the TUI

The TUI has been enhanced to display ICMP extensions in both the normal and hop detail navigation modes.

In normal mode, ICMP extensions are not shown by default but can be enabled by setting the `--tui-icmp-extension-mode` command line flag or by adding the `tui-icmp-extension-mode` entry in the `tui` section of the configuration file:

```toml
[tui]
tui-icmp-extension-mode = "full"
```

This can be set to `off` (do not show ICMP extension data), `mpls` (shows a list of MPLS label(s) per hop), `full` (shows all details of all extensions, such as `ttl`, `exp` and `bos` for MPLS) or `all` (the same as `full` but also shows `class`, `subtype` and `bytes` for unknown extension objects).

The following screenshot shows ICMP extensions in normal mode with `tui-icmp-extension-mode` set to be `mpls`:

<img width="60%" alt="extensions" src="https://github.com/fujiapple852/trippy/blob/master/assets/0.9.0/extensions.png">

In hop detail mode, the full details of all ICMP extension objects are always shown if parsing of ICMP extensions is enabled.

The following screenshot shows ICMP extensions in hop detail mode:

<img width="60%" alt="extensions_detail" src="https://github.com/fujiapple852/trippy/blob/master/assets/0.9.0/extensions_detail.png">

#### ICMP Extensions in Reports

ICMP extension information is also included the the `json` and `stream` report modes.

Sample output for a single hop from the `json` report:

```json
{
  "ttl": 14,
  "hosts": [
    {
      "ip": "129.250.3.125",
      "hostname": "ae-4.r25.sttlwa01.us.bb.gin.ntt.net"
    }
  ],
  "extensions": [
    {
      "mpls": {
        "members": [
          {
            "label": 91106,
            "exp": 0,
            "bos": 1,
            "ttl": 1
          }
        ]
      }
    }
  ],
  "loss_pct": "0.00",
  "sent": 1,
  "last": "178.16",
  "recv": 1,
  "avg": "178.16",
  "best": "178.16",
  "worst": "178.16",
  "stddev": "0.00"
}
```

### Paris Tracing Strategy for IPv6/UDP

The work to support the remaining [paris](https://github.com/libparistraceroute/libparistraceroute/wiki/Checksum) and [dublin](https://github.com/insomniacslk/dublin-traceroute) tracing modes continues in this release with the addition of support for the Paris tracing strategy for IPv6/UDP.

As a reminder, unlike classic traceroute and MTR, these alternative tracing strategies do not encode the probe sequence number in either the src or dest port of the UDP or TCP packet, but instead use other protocol and address family specific techniques. Specifically, the Paris tracing strategy for IPv6/UDP utilizes the UDP checksum for this purposes and manipulates the UDP payload to ensure packets remind valid.

By doing so, these strategies are able to keep the src and dest ports fixed which makes it much more likely (though not guaranteed) that each round of tracing will follow the same path through the network (note that this is _not_ true for the return path).

The following command runs a IPv6/UDP trace using the `paris` tracing strategy with fixed src and dest ports:

```shell
trip example.com --udp -6 -R paris -S 5000 -P 3500
```

Refer to the [tracking issue](https://github.com/fujiapple852/trippy/issues/274) for details of the work remaining to support all ECMP strategies for both UDP and TCP for IPv4 and IPv6.

### Unprivileged Mode

Trippy normally requires elevated privileges due to the use of raw sockets. Enabling the required privileges for a given platform can be achieved in several ways as in described the [privileges](https://github.com/fujiapple852/trippy#privileges) section of the documentation.

This release of Trippy adds the ability to run _without_ elevated privileged on a subset of platforms, but with some limitations which are described below.

The unprivileged mode can be enabled by adding the `--unprivileged` (`-u`) command line flag or by adding the `unprivileged` entry in the `trippy` section of the configuration file:

```toml
[trippy]
unprivileged = true
```

The following command runs a trace in unprivileged mode:

```shell
trip example.com -u
```

Unprivileged mode is currently only supported on macOS. Linux support is possible and may be added in the future.  Unprivileged mode is not supported on NetBSD, OpenBSD, FreeBSD or Windows as these platforms do not support the `IPPROTO_ICMP` socket type.

Unprivileged mode does not support the `paris` or `dublin` tracing strategies as these require raw sockets in order to manipulate the UDP and IP header respectively.

See [#101](https://github.com/fujiapple852/trippy/issues/101) for further information.

### Resolve All DNS

Trippy can be provided with either an IP address or a hostname as the target for tracing.  Trippy will resolve hostnames to IP addresses via DNS lookup (using the configured DNS resolver, see the existing `--dns-resolve-method` flag) and pick an arbitrary IP address from those returned.

Trippy also has the ability to trace to several targets simultaneously (for the ICMP protocol only) and can be provided with a list of IP addresses and hostnames.

Trippy `0.9.0` combined these features and introduces a convenience flag `--dns-resolve-all` which resolves a given hostname to all IP addresses and will begin to trace to all of them simultaneously.

<img width="60%" alt="dns_resolve_all" src="https://github.com/fujiapple852/trippy/blob/master/assets/0.9.0/dns_resolve_all.png">

### Hop Privacy

At times it is desirable to share tracing information with others to help with diagnostics of a network problem. These traces can contain sensitive information, such as IP addresses, hostnames and GeoIp details of the internet facing hops. Users often wish to avoid exposing this data and are forced to redact the tracing output or screenshots.

Trippy `0.9.0` adds a new privacy feature, which hides all sensitive information for a configurable number of hops in the hops table, chart and GeoIP world map.

The following screenshot shows the world map view with the sensitive information of some hops hidden:

<img width="60%" alt="privacy" src="https://github.com/fujiapple852/trippy/blob/master/assets/0.9.0/privacy.png">

The following command will hide all sensitive information for the first 3 hops (ttl 1, 2 & 3) in the TUI:

```shell
trip example.com --tui-privacy-max-ttl 3
```

This can also be made the default behaviour by setting the value in the Trippy configuration file:

```toml
[tui]
tui-privacy-max-ttl = 3
```

From within the TUI the privacy mode can be toggled on and off using the `toggle-privacy` TUI command (bound to the `p`key by default).

Note the toggle is only available if `tui-privacy-max-ttl` is configured to be non-zero.  Privacy mode is entered automatically on startup to avoid any accidental exposure of sensitive data, such as when sharing a screen.

### Print Config Template

The `0.8.0` release of Trippy introduced a [configuration file](https://github.com/fujiapple852/trippy#configuration-reference) and provided a sample configuration file you could download. This release adds a command which generates a configuration template appropriate for the specific version of Trippy.

The following command generates a `trippy.toml` configuration file with all possible configuration options specified and set to their default values:

```shell
trip --print-config-template > trippy.toml
```

### Alternative Help Key Binding

Can't decide whether you want to use `h` or `?` to display help information? Well fear not, Trippy now supports an `toggle-help-alt` TUI command (bound to the `?` key by default) in additional to the existing `toggle-help` TUI command (bound to the `h` key by default).

### Improvements to Reports

This release fixes a bug that prevented reverse DNS lookup from working in all reporting modes.

The list of IPs associated with a given hop have also been added to the `csv` and all tabular reports.  ICMP extension data has also been included in several reports.

Note that these are breaking change as the output of the reports has changed.

### New Binary Asset Downloads

The list of operating systems, CPU architectures and environments which have pre-build binary assets available for download has been greatly expanded for the `0.9.0` release.

This includes assets for Linux, macOS, Windows, NetBSD and FreeBSD. Assets are available for `x86_64`, `aarch64` and `arm7` and includes builds for various environments such as `gnu` and `musl` where appropriate. There are also pre-build `RPM` and `deb` downloads available. See the [Binary Asset Download](https://github.com/fujiapple852/trippy#binary-asset-download) section for a full list.

Note that Trippy `0.9.0` has only been [tested](https://github.com/fujiapple852/trippy/issues/836) on a small subset of these platforms.

### New Distribution Packages

Since the last release Trippy has been added as an official WinGet package (kudos to @mdanish-kh and @BrandonWanHuanSheng!) and can be installed as follows:

```shell
winget install trippy
```

Trippy has also been added to the scoop `Main` bucket (thanks to @StarsbySea!) and can be installed as follows:

```shell
scoop install trippy
```

You can find the full list of [distributions](https://github.com/fujiapple852/trippy/tree/master#distributions) in the documentation.

### Thanks

My thanks to all Trippy contributors, package maintainers and community members.

Feel free to drop by the Trippy Matrix room for a chat:

[![#trippy-dev:matrix.org](https://img.shields.io/badge/matrix/trippy-dev:matrix.org-blue)](https://matrix.to/#/#trippy-dev:matrix.org)

Happy Tracing!

## New Contributors
* @c-git made their first contribution in https://github.com/fujiapple852/trippy/pull/632
* @trkelly23 made their first contribution in https://github.com/fujiapple852/trippy/pull/788

# 0.8.0

## Highlights

The `0.8.0` release of Trippy brings several new features, UX enhancements, and quality of life improvements, as well as various small fixes and other minor improvements.

#### Hop Detail Navigation

Trippy offers various mechanisms to visualize [ECMP](https://en.wikipedia.org/wiki/Equal-cost_multi-path_routing) (Equal-Cost Multi-Path Routing) when tracing with UDP and TCP protocols. Features include displaying all hosts for a given hop in a scrollable table, limiting the number of hosts shown per hop (showing the % of traffic for each host), and greying out hops that are not part of a specific tracing round.

Despite these helpful features, visualizing a complete trace can be challenging when there are numerous hosts for some hops, which is common in environments where ECMP is heavily utilized.

This release enhances ECMP visualization support by introducing a hop detail navigation mode, which can be toggled on and off by pressing `d` (default key binding). This mode displays multiline information for the selected hop only, including IP, hostname, AS, and GeoIP details about a single host for the hop. Users can navigate forward and backward between hosts in a given hop by pressing `,` and `.` (default key bindings), respectively.

<img src="https://github.com/fujiapple852/trippy/blob/master/assets/0.8.0/hop_details.png" width="60%">

In addition to visualizing ECMP, Trippy also supports alternative tracing strategies to assist with ECMP routing, which are described below.

#### Paris Tracing Strategy

Trippy already supports both classic and [dublin](https://github.com/insomniacslk/dublin-traceroute) tracing strategies, and this release adds support for the [paris](https://github.com/libparistraceroute/libparistraceroute/wiki/Checksum) tracing strategy for the UDP protocol.

Unlike classic traceroute and MTR, these alternative tracing strategies do not encode the probe sequence number in either the src or dest port of the UDP or TCP packet, but instead use other protocol and address family specific techniques.

This means that every probe in a trace can share common values for the src & dest hosts and ports which, when combined with the protocol, is typically what is used to making traffic route decisions in ECMP routing.  This means that these alternative tracing strategies significantly increase the likelihood that the same path is followed for each probe in a trace (but not the return path!) in the presence of ECMP routing.

The following command runs a UDP trace using the new `paris` tracing strategy with fixed src and dest ports (the src and dest hosts and the protocol are always fixed) and will therefore likely follow a common path for each probe in the trace:

```shell
trip www.example.com --udp -R paris -S 5000 -P 3500
```

Future Trippy versions will build upon these strategies and further improve the ability to control and visualize ECMP routing, refer to the [tracking issue](https://github.com/fujiapple852/trippy/issues/274) for further details.

#### GeoIp Information & Interactive Map

Trippy now supports the ability to look up and display GeoIP information from a user-provided MaxMind [GeoLite2 City database](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data).  This information is displayed per host in the hop table (for both normal and new detail navigation modes) and can be shown in various formats. For example, short form like "San Jose, CA, US" or long form like "San Jose, California, United States, North America," or latitude, longitude, and accuracy radius like "37.3512, -121.8846 (~20km)".

The following command enables GeoIP lookup from the provided `GeoLite2-City.mmdb` file and will show long form locations in the hop table:

```shell
trip example.com --geoip-mmdb-file GeoLite2-City.mmdb --tui-geoip-mode long
```

Additionally, Trippy features a new interactive map screen that can be toggled on and off by pressing `m` (default key binding). This screen displays a world map and plots the location of all hosts for all hops in the current trace, as well as highlighting the location of the selected hop.

<img src="https://github.com/fujiapple852/trippy/blob/master/assets/0.8.0/world_map.png" width="60%">

#### Autonomous System Display Enhancements

Trippy has long offered the ability to look up and display AS information. This release makes this feature more flexible by allowing different AS details to be shown in the hops table, including AS number, AS name, prefix CIDR, and registry details.

<img src="https://github.com/fujiapple852/trippy/blob/master/assets/0.8.0/as_info.png" width="60%">

The following command enables AS lookup and will display the prefix CIDR for each host in the TUI:

```shell
trip example.com -z true -r resolv --tui-as-mode prefix
```

This release also fixes a limitation in earlier versions of Trippy that prevented the lookup of AS information for IP addresses without a corresponding `PTR` DNS record.

#### UI Cleanup & Configuration Dialog

The number of configurable parameters in Trippy has grown significantly, surpassing the number that can be comfortably displayed in the TUI header section. Previous Trippy versions displayed an arbitrarily chosen subset of these parameters, many of which have limited value for users and consume valuable screen space.

This release introduces a new interactive settings dialog that can be toggled on and off with `s` (default key binding) to display all configured parameters. The TUI header has also been cleaned up to show only the most relevant information, specifically the protocol and address family, the AS info toggle, the hop details toggle, and the max-hosts setting.

<img src="https://github.com/fujiapple852/trippy/blob/master/assets/0.8.0/settings.png" width="60%">

#### Configuration File

The previous Trippy release introduced the ability to customize the TUI color theme and key bindings, both of which could be specified by command-line arguments. While functional, this method is inconvenient when configuring a large number of colors or keys.

This release adds support for a Trippy configuration file, allowing for persistent storage of color themes, key bindings, and all other configuration items supported by Trippy.

For a sample configuration file showing all possible configurable items that are available, see the [configuration reference](https://github.com/fujiapple852/trippy#configuration-reference) for details.

#### Shell Completions

This release enables the generation of shell completions for various shells, including bash, zsh, PowerShell, and fish, using the new `--generate` command-line flag.

The following command will generate and store shell completions for the fish shell:

```shell
trip --generate fish > ~/.config/fish/completions/trip.fish
```

#### Improved Error Reporting & Debug Logging

This release adds a number of command-line flags to enable debug logging, enhancing the ability to diagnose failures. For example, the following command can be used to run tracing with no output, except for debug output in a format suitable to be displayed with `chrome://tracing` or similar tools:

```shell
trip www.example.com -m silent -v --log-format chrome
```

Socket errors have also been augmented with contextual information, such as the socket address for a bind failure, to help with the diagnosis of issues.

#### New Distribution Packages

Trippy is now also available as a Nix package (@figsoda), a FreeBSD port (@ehaupt) and a Windows Scoop package.  This release also reenables support for a `musl` binary which was disabled in `0.7.0` due to a bug in a critical library used by Trippy.

See [distributions](https://github.com/fujiapple852/trippy#distributions) for the full list of available packages.

My thanks, as ever, to all Trippy contributors!

## New Contributors
* @utkarshgupta137 made their first contribution in https://github.com/fujiapple852/trippy/pull/537

# 0.7.0

## Highlights

The major highlight of the 0.7.0 release of Trippy is the addition of full support for Windows, for all tracing modes and protocols! 🎉.  This has been many months in the making and is thanks to the hard work and perseverance of @zarkdav.

This release also sees the introduction of custom Tui themes and key bindings, `deb` and `rpm` package releases, as well as several important bug fixes.

My thanks to all the contributors!

# 0.6.0

## Highlights

The first official release of Trippy!