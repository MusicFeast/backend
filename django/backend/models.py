from email.mime import image
from email.policy import default
from django.db import models

# Create your models here.

class Carousel(models.Model):
    image=models.ImageField(null=True, blank=True)
    is_visible=models.BooleanField(default=False)

class Artist(models.Model):
    name=models.CharField(max_length=255,null=False,blank=False, default="")
    description=models.CharField(max_length=255,null=False,blank=False, default="")
    about=models.CharField(max_length=255,null=False,blank=False, default="")
    image=models.ImageField(null=True, blank=True)
    is_visible=models.BooleanField(default=False)
    comming=models.BooleanField(default=False)
    instagram=models.CharField(max_length=255,null=True,blank=True)
    twitter=models.CharField(max_length=255,null=True,blank=True)
    facebook=models.CharField(max_length=255,null=True,blank=True)
    discord=models.CharField(max_length=255,null=True,blank=True)

class Home(models.Model):
    artist=models.OneToOneField(Artist,on_delete=models.CASCADE)

class News(models.Model):
    title=models.CharField(max_length=255,null=False,blank=False, default="")
    title2=models.CharField(max_length=255,null=False,blank=False, default="")
    description=models.CharField(max_length=255,null=False,blank=False, default="")
    desc_long=models.CharField(max_length=255,null=False,blank=False, default="")
    is_visible=models.BooleanField(default=False)