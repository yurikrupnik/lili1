You are an AI assistant tasked with creating a custom command for Claude that interacts with GitHub. This command will commit changes to a repository, push those changes, and create a pull request if one doesn't already exist. If a pull request already exists, it will simply log the information about the existing pull request.

You will be provided with the following input variables:
<github_repo>{{GITHUB_REPO}}</github_repo>
This is the full name of the GitHub repository (e.g., "username/repo-name").

<branch_name>{{BRANCH_NAME}}</branch_name>
This is the name of the branch where the changes will be committed and pushed.

<commit_message>{{COMMIT_MESSAGE}}</commit_message>
This is the message that will be used for the commit.

<file_paths>{{FILE_PATHS}}</file_paths>
This is a comma-separated list of file paths that have been modified and need to be committed.

Follow these steps to complete the task:

1. Check if the provided GitHub repository exists and is accessible. If not, return an error message.

2. Switch to the specified branch. If the branch doesn't exist, create it.

3. Stage the files specified in the file_paths variable.

4. Commit the changes using the provided commit message.

5. Push the changes to the remote repository.

6. Check if a pull request already exists for this branch:
   a. If a pull request doesn't exist, create a new pull request with the following details:
  - Title: Use the commit message as the PR title
  - Body: "Automated pull request created by Claude"
  - Base branch: main (or master, depending on the repository's default branch)
  - Compare branch: the branch specified in branch_name
    b. If a pull request already exists, retrieve its information (PR number, title, and URL).

7. Provide output based on the result:
  - If a new pull request was created, output the PR number, title, and URL.
  - If a pull request already existed, output a message indicating this, along with the existing PR's number, title, and URL.

Handle potential errors gracefully, such as:
- Invalid repository name
- Insufficient permissions
- Network issues
- Merge conflicts

Output your response in the following format:

<result>
[Success or Error message]
[Pull Request Information or Error Details]
</result>

Example successful output:
<result>
Success: Changes committed and pushed to branch 'feature-branch'
New Pull Request created:
PR #42: "Add new feature"
URL: https://github.com/username/repo-name/pull/42
</result>

Example output for existing PR:
<result>
Success: Changes committed and pushed to branch 'feature-branch'
Existing Pull Request found:
PR #37: "Update documentation"
URL: https://github.com/username/repo-name/pull/37
</result>

Example error output:
<result>
Error: Unable to access repository
Details: The specified repository 'username/repo-name' does not exist or you don't have the necessary permissions.
</result>
