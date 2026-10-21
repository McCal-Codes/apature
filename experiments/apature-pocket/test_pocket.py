import json
import tempfile
import unittest
from pathlib import Path
from pocket import Receiver, append_jsonl, consume_events, records, require_name


class PocketTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.spool = self.root / 'camera'
        self.spool.mkdir()
        self.receiver = Receiver(self.root / 'data', 'test_20261020', 'Nikon_Z6II', self.spool)

    def test_copy_sha_manifest_and_source_preserved(self):
        file = self.spool / 'DSC_0001.NEF'
        file.write_bytes(b'fake NEF bytes' * 100)
        self.assertEqual(self.receiver.ingest_one(file), 'copied')
        target = self.receiver.archive / file.name
        self.assertEqual(file.read_bytes(), target.read_bytes())
        self.assertEqual(self.receiver.ingest_one(file), 'already-verified')
        rows = list(records(self.receiver.manifest))
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['source_verified'], 'spool-only; camera originals not verified')

    def test_different_camera_file_same_name_refuses_collision(self):
        original = self.spool / 'DSC_0001.NEF'
        original.write_bytes(b'first')
        self.receiver.ingest_one(original)
        original.write_bytes(b'second')
        with self.assertRaisesRegex(RuntimeError, 'changed after ingest'):
            self.receiver.ingest_one(original)
        self.assertEqual((self.receiver.archive / original.name).read_bytes(), b'first')

    def test_unknown_prior_destination_refused(self):
        file = self.spool / 'DSC_0002.NEF'
        file.write_bytes(b'bytes')
        self.receiver.ingest_one(file)
        (self.receiver.archive / file.name).unlink()
        with self.assertRaisesRegex(RuntimeError, 'missing or corrupt'):
            self.receiver.ingest_one(file)

    def test_spoofed_outside_input_rejected(self):
        file = self.root / 'out.NEF'
        file.write_bytes(b'x')
        with self.assertRaises(ValueError):
            self.receiver.ingest_one(file)

    def test_queue_hook_event_replay_idempotent(self):
        file = self.spool / 'DSC_0042.JPG'
        file.write_bytes(b'JPEG bytes')
        queue = self.root / 'queue.jsonl'
        append_jsonl(queue, {'path': str(file)})
        self.assertEqual(consume_events(queue, self.receiver), 1)
        self.assertEqual(consume_events(queue, self.receiver), 0)
        self.assertTrue((self.receiver.archive / file.name).exists())

    def test_recovery_after_copy_before_manifest(self):
        file = self.spool / 'DSC_1234.NEF'
        file.write_bytes(b'abcd')
        (self.receiver.archive / file.name).write_bytes(b'abcd')
        self.assertEqual(self.receiver.ingest_one(file), 'recovered-identical')
        self.assertEqual(len(list(records(self.receiver.manifest))), 1)

    def test_partial_json_line_ignored(self):
        path = self.root / 'events.jsonl'
        path.write_bytes(b'{"path":"ok"}\n{"path":')
        self.assertEqual(list(records(path)), [{'path': 'ok'}])

    def test_unsupported_extensions_and_traversal_names(self):
        file = self.spool / 'notes.exe'
        file.write_bytes(b'data')
        with self.assertRaises(ValueError):
            self.receiver.ingest_one(file)
        with self.assertRaises(ValueError):
            require_name('../bad')

    def test_pair_preserved(self):
        for name in ('DSC_10.NEF', 'DSC_10.JPG'):
            (self.spool / name).write_bytes(name.encode())
        counts = self.receiver.ingest_folder()
        self.assertEqual(counts['copied'], 2)
        self.assertEqual({p.name for p in self.receiver.archive.iterdir()}, {'DSC_10.NEF', 'DSC_10.JPG'})


if __name__ == '__main__':
    unittest.main()

