# Generated by Django 4.0.4 on 2022-10-05 20:25

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('backend', '0005_news_remove_artist_is_home_artist_about_and_more'),
    ]

    operations = [
        migrations.AlterField(
            model_name='artist',
            name='discord',
            field=models.CharField(blank=True, max_length=255, null=True),
        ),
        migrations.AlterField(
            model_name='artist',
            name='facebook',
            field=models.CharField(blank=True, max_length=255, null=True),
        ),
        migrations.AlterField(
            model_name='artist',
            name='instagram',
            field=models.CharField(blank=True, max_length=255, null=True),
        ),
        migrations.AlterField(
            model_name='artist',
            name='twitter',
            field=models.CharField(blank=True, max_length=255, null=True),
        ),
    ]