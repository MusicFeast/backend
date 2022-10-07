from rest_framework import serializers
from .models import *
from django.contrib.auth.models import User

class CarouselSerializer(serializers.ModelSerializer):
    class Meta:
        model = Carousel
        fields = '__all__'  

class ArtistSerializer(serializers.ModelSerializer):
    class Meta:
        model = Artist
        fields = '__all__'  

class HomeSerializer(serializers.ModelSerializer):
    class Meta:
        model = Home
        fields = '__all__'  

class NewsSerializer(serializers.ModelSerializer):
    class Meta:
        model = News
        fields = '__all__'  