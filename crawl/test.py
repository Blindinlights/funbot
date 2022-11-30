import github
url = 'https://www.zhihu.com/question/569763448'
res=github.get_github_repo_property(url)
print(res)