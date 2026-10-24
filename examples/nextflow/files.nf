#!/usr/bin/env nextflow
// Local file-producing acceptance fixture; not upstream Nextflow source.
params.delay = 3

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
    sleep ${params.delay}
    tr '[:lower:]' '[:upper:]' < '${source}' > '${name}.result.txt'
    """
}

workflow {
    FINISH(PREPARE(channel.of('alpha', 'beta')))
}
