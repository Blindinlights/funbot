import github
url = 'https://github.com/Blindinlights-cc/funbot'
res=github.get_github_repo_property(url)
print(res)