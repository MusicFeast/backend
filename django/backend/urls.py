from django.urls import path, include
from rest_framework import routers

from django.urls import path, include
from rest_framework import routers
from .views import *


router = routers.DefaultRouter()
router.register(r'carousel', CarouselVS,basename='carousel')
router.register(r'artist', ArtistVS,basename='artist')
router.register(r'home', HomeVS,basename='home')
router.register(r'news', NewsVS,basename='news')

urlpatterns = [
    path('', include(router.urls)),
    path('get-carousel', get_carousel),
    path('get-artists', get_artists),
    path('get-artists-home', get_artists_home),
    path('get-news', get_news)
]