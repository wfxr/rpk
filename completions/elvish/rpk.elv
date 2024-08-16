
use builtin;
use str;

set edit:completion:arg-completer[rpk] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'rpk'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'rpk'= {
            cand --color 'This flag controls when to use colors'
            cand --config-dir 'The configuration directory'
            cand --data-dir 'The directory to store package data'
            cand --cache-dir 'The directory to store downloaded packages'
            cand --bin-dir 'The directory installed binaries linked to'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand init 'Initialize a configuration file'
            cand list 'List all installed packages'
            cand l 'List all installed packages'
            cand ls 'List all installed packages'
            cand sync 'Install any missing packages, re-generating the lock file'
            cand s 'Install any missing packages, re-generating the lock file'
            cand add 'Add a new plugin to the config file'
            cand a 'Add a new plugin to the config file'
            cand restore 'Restore packages to the state in the lockfile'
            cand r 'Restore packages to the state in the lockfile'
            cand update 'Update packages and re-generate the lock file'
            cand u 'Update packages and re-generate the lock file'
            cand find 'Find packages matching the given query'
            cand f 'Find packages matching the given query'
            cand fd 'Find packages matching the given query'
            cand cleanup 'Remove packages which are not listed in the lock file'
            cand env 'Prints the environment variables for rpk'
            cand completions 'Generate completions for the given shell'
            cand version 'Prints detailed version information'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'rpk;init'= {
            cand -f 'The config file URL to initialize from'
            cand --from 'The config file URL to initialize from'
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;list'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;l'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;ls'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;sync'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;s'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;add'= {
            cand --name 'A unique name for the package. Defaults to the repo name'
            cand --version 'The version of the package'
            cand --desc 'A description of the package'
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'rpk;a'= {
            cand --name 'A unique name for the package. Defaults to the repo name'
            cand --version 'The version of the package'
            cand --desc 'A description of the package'
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'rpk;restore'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;r'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;update'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;u'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;find'= {
            cand --top 'The number of results to display'
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;f'= {
            cand --top 'The number of results to display'
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;fd'= {
            cand --top 'The number of results to display'
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;cleanup'= {
            cand --color 'This flag controls when to use colors'
            cand --cache 'Remove all cached data as well'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;env'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;completions'= {
            cand -d 'The directory to write the completions to'
            cand --dir 'The directory to write the completions to'
            cand --color 'This flag controls when to use colors'
            cand -l 'List all available shells'
            cand --list 'List all available shells'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'rpk;version'= {
            cand --color 'This flag controls when to use colors'
            cand -q 'Suppress any informational output'
            cand --quiet 'Suppress any informational output'
            cand -v 'Use verbose output'
            cand --verbose 'Use verbose output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;help'= {
            cand init 'Initialize a configuration file'
            cand list 'List all installed packages'
            cand sync 'Install any missing packages, re-generating the lock file'
            cand add 'Add a new plugin to the config file'
            cand restore 'Restore packages to the state in the lockfile'
            cand update 'Update packages and re-generate the lock file'
            cand find 'Find packages matching the given query'
            cand cleanup 'Remove packages which are not listed in the lock file'
            cand env 'Prints the environment variables for rpk'
            cand completions 'Generate completions for the given shell'
            cand version 'Prints detailed version information'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'rpk;help;init'= {
        }
        &'rpk;help;list'= {
        }
        &'rpk;help;sync'= {
        }
        &'rpk;help;add'= {
        }
        &'rpk;help;restore'= {
        }
        &'rpk;help;update'= {
        }
        &'rpk;help;find'= {
        }
        &'rpk;help;cleanup'= {
        }
        &'rpk;help;env'= {
        }
        &'rpk;help;completions'= {
        }
        &'rpk;help;version'= {
        }
        &'rpk;help;help'= {
        }
    ]
    $completions[$command]
}
