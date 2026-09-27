import { readFileSync, writeFileSync } from 'node:fs';

const [version, url] = process.argv.slice(2);

const manifest = {
  version,
  notes: readFileSync('CHANGELOG.md', 'utf8'),
  pub_date: new Date().toISOString(),
  platforms: {
    'windows-x86_64': { signature: readFileSync(`ordo-${version}.exe.sig`, 'utf8').trim(), url },
  },
};

writeFileSync('latest.json', JSON.stringify(manifest, null, 2));
