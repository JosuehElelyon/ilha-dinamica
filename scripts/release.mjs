// Publica uma nova versão: atualiza o número em todos os arquivos, faz commit,
// cria a tag e envia. O GitHub Actions compila e as ilhas instaladas se atualizam.
// Uso: npm run release 2.1.1
import { readFileSync, writeFileSync } from 'node:fs'
import { execSync } from 'node:child_process'

const v = process.argv[2]
if (!/^\d+\.\d+\.\d+$/.test(v || '')) {
  console.error('Uso: npm run release 2.1.1')
  process.exit(1)
}
const sh = (c) => execSync(c, { stdio: 'inherit' })

const json = (p, fn) => { const o = JSON.parse(readFileSync(p, 'utf8')); fn(o); writeFileSync(p, JSON.stringify(o, null, 2) + '\n') }
json('package.json', (o) => { o.version = v })
json('src-tauri/tauri.conf.json', (o) => { o.version = v })
const cargo = 'src-tauri/Cargo.toml'
writeFileSync(cargo, readFileSync(cargo, 'utf8').replace(/^version = ".*"$/m, `version = "${v}"`))

sh('npm install --package-lock-only --no-audit --no-fund')
sh(`git add package.json package-lock.json src-tauri/tauri.conf.json src-tauri/Cargo.toml`)
sh(`git commit -m "Versão ${v}"`)
sh(`git tag v${v}`)
sh('git push')
sh(`git push origin v${v}`)
console.log(`\nVersão ${v} enviada! Acompanhe em https://github.com/JosuehElelyon/ilha-dinamica/actions`)
