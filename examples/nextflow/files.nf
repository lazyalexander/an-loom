#!/usr/bin/env nextflow
// Local file-producing acceptance fixture; not upstream Nextflow source.
params.delay = 3
params.pause_after_first = false

process PREPARE {
    input:
    val name
    output:
    tuple val(name), path("${name}.txt")
    script:
    """
    printf '%s\\n' '${name}' > '${name}.txt'
    """
}

process FINISH {
    maxForks 1
    publishDir 'results', mode: 'copy'
    input:
    tuple val(name), path(source)
    output:
    path("${name}.result.txt")
    script:
    """
    if [ '${params.pause_after_first}' = true ] &&
       ! mkdir '${workflow.launchDir}/first-finish' 2>/dev/null &&
       mkdir '${workflow.launchDir}/cancel-ready' 2>/dev/null; then
        # The host deadline bounds this wait. The explicit successor retains the witness.
        exec sleep 300
    fi
    sleep ${params.delay}
    tr '[:lower:]' '[:upper:]' < '${source}' > '${name}.result.txt'
    """
}

workflow {
    FINISH(PREPARE(channel.of('alpha', 'beta')))
}
