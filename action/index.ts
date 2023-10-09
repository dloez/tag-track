import * as core from "@actions/core"

const actionName = process.env.GITHUB_ACTION || ""
const actionPath = process.env.GITHUB_ACTION_PATH || ""
const actionRepository = process.env.GITHUB_ACTION_REPOSITORY || ""

core.debug(actionName)
core.debug(actionPath)
core.debug(actionRepository)
