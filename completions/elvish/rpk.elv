
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
            cand env 'print environment information'
            cand sync 'install any missing packages, re-generating the lock file'
            cand add 'Add a new plugin to the config file'
            cand restore 'Restore packages to the state in the lockfile'
            cand update 'Update packages and re-generate the lock file'
            cand search 'Search for packages on GitHub'
            cand completions 'Generate completions for the given shell'
            cand version 'Prints detailed version information'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'rpk;env'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;sync'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;add'= {
            cand --name 'A unique name for the package'
            cand --version 'The version of the package'
            cand --desc 'A description of the package'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'rpk;restore'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;update'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;search'= {
            cand --top 'The number of results to display'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;completions'= {
            cand -d 'The directory to write the completions to'
            cand --dir 'The directory to write the completions to'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'rpk;version'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'rpk;help'= {
            cand env 'print environment information'
            cand sync 'install any missing packages, re-generating the lock file'
            cand add 'Add a new plugin to the config file'
            cand restore 'Restore packages to the state in the lockfile'
            cand update 'Update packages and re-generate the lock file'
            cand search 'Search for packages on GitHub'
            cand completions 'Generate completions for the given shell'
            cand version 'Prints detailed version information'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'rpk;help;env'= {
        }
        &'rpk;help;sync'= {
        }
        &'rpk;help;add'= {
        }
        &'rpk;help;restore'= {
        }
        &'rpk;help;update'= {
        }
        &'rpk;help;search'= {
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
