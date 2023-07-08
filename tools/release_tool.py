import argparse
import configparser
import datetime
import io
import re
import subprocess
import sys
import time
import zoneinfo

import semver
import tomlkit


def log(message, command=False):
    prefix = "$" if command else "#"
    print(f"{prefix} {message}", file=sys.stderr)


def run_command(description, args, capture_output=False, shell=True):
    if description:
        log(description)
    printed_args = args.join(" ") if type(args) == list else args
    log(printed_args, command=True)
    stdout = subprocess.PIPE if capture_output else None
    completed_process = subprocess.run(args, stdout=stdout, shell=shell, check=True, encoding="utf-8")
    return completed_process.stdout.rstrip() if capture_output else None


def run_command_with_retries(description, args, timeout, backoff, capture_output=False, shell=True):
    start = time.time()
    wait = 1.0
    while True:
        try:
            return run_command(description, args, capture_output=capture_output, shell=shell)
        except Exception as e:
            elapsed = time.time() - start
            if elapsed >= timeout:
                raise e
        w = min(wait, timeout - elapsed)
        log(f"Retrying in {w:.1f} seconds")
        time.sleep(w)
        wait *= backoff


def get_version(version_str=None):
    if not version_str:
        with open("VERSION") as f:
            version_str = f.read().strip()
    return semver.Version.parse(version_str)


def init_channel(args):
    log(f"*** INITIALIZING CHANNEL FROM UPSTREAM ***")
    channel_to_upstream = {"nightly": "origin/main", "beta": "origin/nightly", "stable": "origin/beta"}
    if args.channel not in channel_to_upstream:
        raise Exception(f"Unknown channel {args.channel}")
    upstream = channel_to_upstream[args.channel] if args.upstream is None else args.upstream
    log(f"Initializing {args.channel} <= {upstream}")
    if args.preserve:
        # We're preserving channel history, so we need to do a merge.
        # git doesn't have a "theirs" merge strategy, so we have to simulate it.
        # Technique from https://stackoverflow.com/a/4912267
#        run_command(f"Fetching channel", f"git fetch origin {args.channel}:{args.channel}")
        run_command(f"Creating temporary branch based on upstream", f"git switch -c tmp {upstream}")
        run_command(f"Merging channel (keeping all upstream)", f"git merge -s ours {args.channel}")
        run_command(f"Switching to channel", f"git switch {args.channel}")
        run_command(f"Merging temporary branch", f"git merge tmp")
        run_command(f"Deleting temporary branch", f"git branch -D tmp")
    else:
        # We're not preserving channel history, so we can just reset the branch.
        run_command(f"Resetting channel to upstream", f"git switch -C {args.channel} {upstream}")


def update_file(path, load, munge, dump, binary=False):
    log(f"Updating {path}")
    b = "b" if binary else ""
    with open(path, f"r{b}") as f:
        data = load(f)
    new_data = munge(data)
    with open(path, f"w{b}") as f:
        dump(new_data, f)


def get_python_version(version):
    # Python (PEP 440) has several annoying quirks that make it not quite compatible with semantic versioning:
    # 1. Python doesn't allow arbitrary tags, only (a|b|rc|post|dev). (You can use (alpha|beta|c|pre|preview|rev|r),
    #    but they'll be mapped to (a|b|rc|rc|rc|post|post) respectively.)
    #    So "1.2.3-nightly.456" will fail, and "1.2.3-alpha.456" gets mapped to "1.2.3a456" (see #2).
    # 2. Python doesn't allow separators between the main version and the tag, nor within the tag.
    #    So "1.2.3-a.456" gets mapped to "1.2.3a456"
    # 3. HOWEVER, Python treats tags "post" and "dev" differently, and in these cases uses a "." separator between
    #    the main version and the tag (but still doesn't allow separators within the tag).
    #    So "1.2.3-dev.456" gets mapped to "1.2.3.dev456".
    # 4. Python requires that all tags have a numeric suffix, and will assume 0 if none is present.
    #    So "1.2.3-dev" gets mapped to "1.2.3.dev0" (by #3 & #4).
    # We don't use all these variations, only (dev|nightly|beta), but if that ever changes, hopefully we won't
    # have to look at this whole mess again.
    tag_to_py_tag = {
        "nightly": "a",
        "beta": "b",
        "c": "rc",
        "pre": "rc",
        "preview": "rc",
        "rev": "post",
        "r": "post",
    }
    if version.prerelease is not None:
        split = version.prerelease.split(".", 2)
        tag = split[0]
        py_tag = tag_to_py_tag.get(tag, tag)
        date = split[1] if len(split) >= 2 else None
        counter = split[2] if len(split) >= 3 else None
        py_n = f"{date}{counter:>03}" if date and counter else (date if date else "0")
        py_separator = "." if py_tag in ("post", "dev") else ""
    else:
        py_tag = None
        py_n = None
        py_separator = None
    # semver can't represent the rendered Python version, so we generate a string.
    if py_tag is not None:
        return f"{version.major}.{version.minor}.{version.patch}{py_separator}{py_tag}{py_n}"
    else:
        return str(version)


def update_version(version):
    log(f"Updating version references to {version}")

    # Main version file
    with open("VERSION", "w") as f:
        print(version, file=f)

    # cargo versions
    # cargo doesn't like build metadata in dependency references, so we strip that for those.
    stripped_version = version.replace(build=None)
    def munge_cargo_root(toml):
        toml["workspace"]["package"]["version"] = str(version)
        toml["dependencies"]["opendp_derive"]["version"] = str(stripped_version)
        toml["build-dependencies"]["opendp_tooling"]["version"] = str(stripped_version)
        return toml
    update_file("rust/Cargo.toml", tomlkit.load, munge_cargo_root, tomlkit.dump)
    def munge_cargo_opendp_derive(toml):
        toml["dependencies"]["opendp_tooling"]["version"] = str(stripped_version)
        return toml
    update_file("rust/opendp_derive/Cargo.toml", tomlkit.load, munge_cargo_opendp_derive, tomlkit.dump)

    python_version = get_python_version(version)
    def load_config(f):
        config = configparser.RawConfigParser()
        config.read_file(f)
        return config
    def munge_config(config):
        config.set("metadata", "version", str(python_version))
        return config
    update_file("python/setup.cfg", load_config, munge_config, lambda data, f: data.write(f))


def today(args):
    if args.time_zone is not None:
        tz = zoneinfo.ZoneInfo(args.time_zone)
        return datetime.datetime.now(tz).date()
    else:
        return datetime.date.today()


def infer_counter(version, date, args):
    if args.counter:
        return args.counter
    if version.prerelease is None:
        return 1
    match = re.match(fr"^{args.channel}\.(\d+)\.(\d+)", version.prerelease)
    if match is None:
        return 1
    version_date = match.group(1)
    version_counter = match.group(2)
    if not version_date == date.strftime('%Y%m%d'):
        return 1
    return int(version_counter) + 1


def configure_channel(args):
    log(f"*** CONFIGURING CHANNEL ***")
    if args.channel not in ("dev", "nightly", "beta", "stable"):
        raise Exception(f"Unknown channel {args.channel}")
    version = get_version()
    if args.channel == "dev":
        # dev has a bare tag
        version = version.replace(prerelease="dev", build=None)
    elif args.channel in ("nightly", "beta"):
        # nightly/beta have a tag with the date and a counter
        date = today(args)
        counter = infer_counter(version, date, args)
        prerelease = f"{args.channel}.{date.strftime('%Y%m%d')}.{counter}"
        version = version.replace(prerelease=prerelease, build=None)
    elif args.channel == "stable":
        # stable has no tag
        version = version.finalize_version()
    update_version(version)


def infer_channel(version):
    if version.prerelease is None:
        return "stable"
    channel = version.prerelease.split(".", 1)[0]
    if channel not in ("dev", "nightly", "beta"):
        raise Exception(f"Unable to infer channel from version {version}")
    return channel


def first_match(lines, pattern):
    matcher = re.compile(pattern)
    for i, line in enumerate(lines):
        match = matcher.match(line)
        if match is not None:
            return i, match
    raise Exception("Didn't match pattern in CHANGELOG")


def changelog(args):
    log(f"*** UPDATING CHANGELOG ***")
    version = get_version()
    channel = infer_channel(version)
    stable_date = args.stable_date or today(args)

    log(f"Reading CHANGELOG")
    with open("CHANGELOG.md") as f:
        lines = f.readlines()
    url_base = "https://github.com/opendp/opendp/compare/"
    i, match = first_match(lines, fr"^## \[(\d+\.\d+\.\d+(?:-\S+)?)\]\({re.escape(url_base)}(\S+)\.\.\.\S+\) - \S+$")
    heading_version = semver.Version.parse(match.group(1))
    diff_source = match.group(2)

    if args.prepend:
        if channel != "dev":
            raise Exception("Can only prepend on dev channel")
        # Check that the VERSION file has been bumped above the existing heading version.
        if version.finalize_version() <= heading_version.finalize_version():
            raise Exception(f"Prepending new heading, but VERSION {version} hasn't been bumped above heading version {heading_version}")
        new_heading_version = heading_version.finalize_version()
        diff_target = f"v{heading_version.finalize_version()}"
        date = stable_date.isoformat()
    else:
        # Check that the VERSION file matches the existing heading version.
        if version.finalize_version() != heading_version.finalize_version():
            raise Exception(f"VERSION {version} isn't compatible with heading version {heading_version}")
        new_heading_version = version
        diff_target = f"v{version}" if channel == "stable" else (channel if channel != "dev" else "HEAD")
        date = stable_date.isoformat() if channel != "dev" else "TBD"

    log(f"Updating heading to {new_heading_version}, {diff_source}...{diff_target}, {date}")
    lines[i] = f"## [{new_heading_version}]({url_base}{diff_source}...{diff_target}) - {date}\n"
    if args.prepend:
        # Prepend a new heading for the current version.
        diff_source = diff_target
        log(f"Prepending new heading for {version}")
        lines[i:i] = [f"## [{version}]({url_base}{diff_source}...HEAD) - TBD\n", "\n", "\n"]

    with open("CHANGELOG.md", "w") as f:
        f.writelines(lines)


def sanity(args):
    log(f"*** RUNNING SANITY TEST ***")
    if args.python_repository not in ("pypi", "testpypi", "local"):
        raise Exception(f"Unknown Python repository {args.python_repository}")
    version = get_version()
    version = get_python_version(version)
    run_command("Creating venv", f"rm -rf {args.venv} && python -m venv {args.venv}")
    if args.python_repository == "local":
        package = f"python/wheelhouse/opendp-{version}-py3-none-any.whl"
        run_command(f"Installing opendp {version}", f". {args.venv}/bin/activate && pip install {package}")
    else:
        index_url = "https://test.pypi.org/simple" if args.python_repository == "testpypi" else "https://pypi.org/simple"
        package = f"opendp=={version}"
        run_command_with_retries(
            f"Installing opendp {version}", f". {args.venv}/bin/activate && pip install -i {index_url} {package}",
            args.package_timeout,
            args.package_backoff
        )
    if args.fake:
        run_command("Running test script", f". {args.venv}/bin/activate && echo FAKE TEST!!!")
    else:
        run_command("Running test script", f". {args.venv}/bin/activate && python tools/test.py")


def bump_version(args):
    log(f"*** BUMPING VERSION NUMBER ***")
    if args.set:
        version = get_version(args.set)
    else:
        if args.position not in ("major", "minor", "patch"):
            raise Exception(f"Unknown position {args.position}")
        version = get_version()
        if args.position == "major":
            version = version.bump_major()
        elif args.position == "minor":
            version = version.bump_minor()
        elif args.position == "patch":
            version = version.bump_patch()
        version = version.replace(prerelease="dev", build=None)
    update_version(version)


def _main(argv):
    parser = argparse.ArgumentParser(description="OpenDP release tool")
    subparsers = parser.add_subparsers(dest="COMMAND", help="Command to run")
    subparsers.required = True

    subparser = subparsers.add_parser("init_channel", help="Initialize the channel")
    subparser.set_defaults(func=init_channel)
    subparser.add_argument("-c", "--channel", choices=["nightly", "beta", "stable"], default="nightly", help="Which channel to target")
    subparser.add_argument("-u", "--upstream", help="Upstream ref")
    subparser.add_argument("-p", "--preserve", dest="preserve", action="store_true", default=False)
    subparser.add_argument("-np", "--no-preserve", dest="preserve", action="store_false")

    subparser = subparsers.add_parser("config_channel", help="Configure the channel")
    subparser.set_defaults(func=configure_channel)
    subparser.add_argument("-c", "--channel", choices=["dev", "nightly", "beta", "stable"], default="dev", help="Which channel to target")
    subparser.add_argument("-z", "--time-zone", help="Time zone for release dates")
    subparser.add_argument("-i", "--counter", type=int, default=0, help="Intra-date version counter")

    subparser = subparsers.add_parser("changelog", help="Update CHANGELOG file")
    subparser.set_defaults(func=changelog)
    subparser.add_argument("-d", "--stable-date", type=datetime.date.fromisoformat, help="Date for next stable release")
    subparser.add_argument("-z", "--time-zone", help="Time zone for release dates (when inferring)")
    subparser.add_argument("-p", "--prepend", dest="prepend", action="store_true", default=False, help="Prepend new empty heading (for dev only)")
    subparser.add_argument("-np", "--no-prepend", dest="prepend", action="store_false", help="Don't prepend new empty heading (for dev only)")

    subparser = subparsers.add_parser("sanity", help="Run sanity test")
    subparser.set_defaults(func=sanity)
    subparser.add_argument("-e", "--venv", default="/tmp/sanity-venv", help="Virtual environment directory")
    subparser.add_argument("-r", "--python-repository", choices=["pypi", "testpypi", "local"], default="pypi", help="Python package repository")
    subparser.add_argument("-t", "--package-timeout", type=int, default=0, help="How long to retry package installation attempts (0 = no retries)")
    subparser.add_argument("-b", "--package-backoff", type=float, default=2.0, help="How much to back off between package installation attempts")
    subparser.add_argument("-f", "--fake", dest="fake", action="store_true", default=False)
    subparser.add_argument("-nf", "--no-fake", dest="fake", action="store_false")

    subparser = subparsers.add_parser("bump_version", help="Bump the version number (assumes dev channel)")
    subparser.set_defaults(func=bump_version)
    subparser.add_argument("-p", "--position", choices=["major", "minor", "patch"], default="minor")
    subparser.add_argument("-s", "--set", help="Set the version to a specific value")

    args = parser.parse_args(argv[1:])
    args.func(args)


def main():
    _main(sys.argv)


if __name__ == "__main__":
    main()
