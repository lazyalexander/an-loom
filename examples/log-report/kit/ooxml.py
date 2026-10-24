"""Document adapter setup: fixed built-in MIME registry, no ambient system files."""
import mimetypes

# Python otherwise probes host Apache MIME files while importing the document
# libraries. Built-ins plus each library's OOXML registrations are sufficient.
# Keep this compatibility detail here; never widen the host sandbox for it.
mimetypes.knownfiles = []
mimetypes.init()
