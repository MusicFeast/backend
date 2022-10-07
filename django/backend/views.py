from copyreg import constructor
from re import search
from rest_framework.authtoken.serializers import AuthTokenSerializer
from rest_framework.authentication import SessionAuthentication,BasicAuthentication,TokenAuthentication
from crypt import crypt
from .serializers import *
from .models import *
from django.contrib.auth import login
import requests
from django_filters import rest_framework as filters
from rest_framework.permissions import IsAdminUser,IsAuthenticated,AllowAny
from rest_framework.response import Response
from rest_framework.authtoken.models import Token
from django.contrib.auth.models import User
from rest_framework import viewsets,status
from rest_framework.decorators import api_view,permission_classes,authentication_classes
from django.views.decorators.csrf import csrf_exempt

# Create your views here.

class CarouselVS(viewsets.ModelViewSet):
    permission_classes=[IsAuthenticated]
    authentication_classes=[TokenAuthentication]
    queryset = Carousel.objects.all()
    serializer_class = CarouselSerializer

@api_view(["GET"])
@csrf_exempt
@authentication_classes([BasicAuthentication])
@permission_classes([AllowAny])
def get_carousel(request):
    carousel = Carousel.objects.filter(is_visible=True)
    serializer = CarouselSerializer(carousel, many=True)
    return Response(serializer.data,status=status.HTTP_200_OK)

class ArtistVS(viewsets.ModelViewSet):
    permission_classes=[IsAuthenticated]
    authentication_classes=[TokenAuthentication]
    queryset = Artist.objects.all()
    serializer_class = ArtistSerializer

@api_view(["GET"])
@csrf_exempt
@authentication_classes([BasicAuthentication])
@permission_classes([AllowAny])
def get_artists(request):
    artists = Artist.objects.filter(is_visible=True)
    serializer = ArtistSerializer(artists, many=True)
    return Response(serializer.data,status=status.HTTP_200_OK)

class HomeVS(viewsets.ModelViewSet):
    permission_classes=[IsAuthenticated]
    authentication_classes=[TokenAuthentication]
    queryset = Home.objects.all()
    serializer_class = HomeSerializer

@api_view(["GET"])
@csrf_exempt
@authentication_classes([BasicAuthentication])
@permission_classes([AllowAny])
def get_artists_home(request):
    home = Home.objects.all()
    artists = HomeSerializer(home, many=True).data
    data = []
    for artist in artists:
        item = Artist.objects.get(id=artist['id'])
        if item.is_visible:
            serializer = ArtistSerializer(item).data
            data.append(serializer)
    return Response(data,status=status.HTTP_200_OK)

class NewsVS(viewsets.ModelViewSet):
    permission_classes=[IsAuthenticated]
    authentication_classes=[TokenAuthentication]
    queryset = News.objects.all()
    serializer_class = NewsSerializer

@api_view(["GET"])
@csrf_exempt
@authentication_classes([BasicAuthentication])
@permission_classes([AllowAny])
def get_news(request):
    news = News.objects.filter(is_visible=True)
    serializer = NewsSerializer(news, many=True)
    return Response(serializer.data,status=status.HTTP_200_OK)