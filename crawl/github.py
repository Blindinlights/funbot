import requests
from bs4 import BeautifulSoup
import json


def get_github_repo_info(repo_url):
    """
    Get the github repo info
    :param repo_url: github repo url
    :return: github repo info
    """
    repo_info = {}
    repo_info['url'] = repo_url
    repo_info['name'] = repo_url.split('/')[-1]
    repo_info['owner'] = repo_url.split('/')[-2]
    repo_info['api_url'] = 'https://api.github.com/repos/{owner}/{repo}'.format(owner=repo_info['owner'],
                                                                                repo=repo_info['name'])
    return repo_info
def get_github_repo_property(repo_url):
    """
    Get the github repo property from <meta>
    :param repo_url: github repo url
    :return: property value
    """
    ret={}
    res=requests.get(repo_url)
    html=res.text
    soup=BeautifulSoup(html,'html.parser')
    metas=soup.find_all('meta')
    for meta in metas:
        if meta.get('property')=='og:title':
            ret['title']=meta.get('content')
        if meta.get('property')=='og:image':
            ret['image']=meta.get('content')
        if meta.get('property')=='og:description':
            ret['description']=meta.get('content')
    title=soup.find('title').text
    ret['title']=title
    return ret
    
    