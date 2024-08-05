
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'rpk' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'rpk'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'rpk' {
            [CompletionResult]::new('--color', 'color', [CompletionResultType]::ParameterName, 'This flag controls when to use colors')
            [CompletionResult]::new('--config-dir', 'config-dir', [CompletionResultType]::ParameterName, 'The configuration directory')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'The directory to store package data')
            [CompletionResult]::new('--cache-dir', 'cache-dir', [CompletionResultType]::ParameterName, 'The directory to store downloaded packages')
            [CompletionResult]::new('--bin-dir', 'bin-dir', [CompletionResultType]::ParameterName, 'The directory installed binaries linked to')
            [CompletionResult]::new('-q', 'q', [CompletionResultType]::ParameterName, 'Suppress any informational output')
            [CompletionResult]::new('--quiet', 'quiet', [CompletionResultType]::ParameterName, 'Suppress any informational output')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Use verbose output')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Use verbose output')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', 'V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('env', 'env', [CompletionResultType]::ParameterValue, 'print environment information')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'install any missing packages, re-generating the lock file')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new plugin to the config file')
            [CompletionResult]::new('restore', 'restore', [CompletionResultType]::ParameterValue, 'Restore packages to the state in the lockfile')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update packages and re-generate the lock file')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for packages on GitHub')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate completions for the given shell')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Prints detailed version information')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rpk;env' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'rpk;sync' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'rpk;add' {
            [CompletionResult]::new('--name', 'name', [CompletionResultType]::ParameterName, 'A unique name for the package')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'The version of the package')
            [CompletionResult]::new('--desc', 'desc', [CompletionResultType]::ParameterName, 'A description of the package')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'rpk;restore' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'rpk;update' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'rpk;search' {
            [CompletionResult]::new('--top', 'top', [CompletionResultType]::ParameterName, 'The number of results to display')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'rpk;completions' {
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'The directory to write the completions to')
            [CompletionResult]::new('--dir', 'dir', [CompletionResultType]::ParameterName, 'The directory to write the completions to')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'rpk;version' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'rpk;help' {
            [CompletionResult]::new('env', 'env', [CompletionResultType]::ParameterValue, 'print environment information')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'install any missing packages, re-generating the lock file')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new plugin to the config file')
            [CompletionResult]::new('restore', 'restore', [CompletionResultType]::ParameterValue, 'Restore packages to the state in the lockfile')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update packages and re-generate the lock file')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for packages on GitHub')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate completions for the given shell')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Prints detailed version information')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rpk;help;env' {
            break
        }
        'rpk;help;sync' {
            break
        }
        'rpk;help;add' {
            break
        }
        'rpk;help;restore' {
            break
        }
        'rpk;help;update' {
            break
        }
        'rpk;help;search' {
            break
        }
        'rpk;help;completions' {
            break
        }
        'rpk;help;version' {
            break
        }
        'rpk;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
