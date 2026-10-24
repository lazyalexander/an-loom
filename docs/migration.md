# Migration and source provenance

The following foundations were ported from the MIT-licensed an-waver source at
revision `1ca1adab36addd15e780c00ac37e1b238f956150`. Its copyright and MIT license
are retained in LICENSE. Loom has no Cargo, runtime, CI checkout or generated
package dependency on an-waver.

| Upstream source | Loom responsibility |
| --- | --- |
| `src/manifest.rs` | Pure identity, inventory, portable path and secret-pattern validation |
| `src/package.rs` | Split by inventory scan, immutable snapshot, archive and local storage responsibilities |
| `src/build_artifacts.rs` | Source-bound reception and closed portable build-record validation |
| `src/spool.rs` | Bounded syntax evidence and fixed file-kit declaration/export adapter |
| `templates/file-list/` | Existing domain example, kept outside generic implementation |
| Package/manifest/reception tests | Migrated refusal and integrity regressions |

Loom adds TOML project bindings, an explicit host development plan, one approved
build submission and a portable local release record. It also drains gzip input
after the tar end marker to verify trailers and reject hidden appended material,
and checks that an existing archive matches the release currently being packed.
These fixes have dedicated regressions. A local-home-path check also refuses
private paths in selected binary payloads; it is incomplete publication hygiene,
not a universal privacy proof.

This first deployment adapter supports `file-kit-v1`. Legacy stdio scaffolds,
model/context command briefs and old command spellings are not advertised as
compatible interfaces. Host-owned model/context commands remain host services.
No model calls or live coding-tool orchestration are claimed by this slice.

The replacement implementation PR has been merged with its required checks
passing. At the maintainer's explicit request, an-waver was made private on
2026-10-24; its history is preserved. It has not been archived. Archive retirement
remains a separate follow-up after checking remaining usage references. The
provenance above remains useful even when a public reader cannot access the old
repository. Loom has no build or runtime dependency on that repository's visibility.
