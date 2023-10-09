import * as core from "@actions/core"
import * as github from "@actions/github"

const context = github.context

core.debug(context.action)
