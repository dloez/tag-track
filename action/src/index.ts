import * as core from '@actions/core'
import * as exec from '@actions/exec'

async function getGitConfigProperty(propertyName: string): Promise<string> {
  const {stdout, stderr} = await exec.getExecOutput('git', [
    'config',
    propertyName
  ])

  if (stderr) {
    core.setFailed('Failed to get git user.name')
  }

  return stdout.trim()
}

async function getCurrentGitAuthor(): Promise<[string, string]> {
  const name = await getGitConfigProperty('user.name')
  const email = await getGitConfigProperty('user.email')

  return [name, email]
}

async function getActionRef(): Promise<string> {
  let ref = ''
  await exec.getExecOutput('git', ['rev-parse', 'HEAD'], {
    cwd: __dirname,
    listeners: {
      stdout: (data: Buffer) => {
        ref += data.toString()
      }
    }
  })

  return ref.trim()
}

async function setupDownloadRun() {
  const runnerOS = process.env.RUNNER_OS
  const runnerArch = process.env.RUNNER_ARCH
  const actionRef = await getActionRef()
  const cacheKey = `tag-track_download${runnerOS}_${runnerArch}_${actionRef}`
  core.debug(`Cache key: ${cacheKey}`)
}

async function run() {
  const {stdout, stderr} = await exec.getExecOutput('ls', [
    '-l'
  ])
  const [name, email] = await getCurrentGitAuthor()
  core.debug(`Git author name: ${name}`)
  core.debug(`Git author email: ${email}`)

  setupDownloadRun()
}

run()
