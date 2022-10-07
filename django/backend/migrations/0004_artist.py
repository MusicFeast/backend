# Generated by Django 4.0.4 on 2022-10-05 17:36

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('backend', '0003_alter_carousel_image'),
    ]

    operations = [
        migrations.CreateModel(
            name='Artist',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('name', models.CharField(default='', max_length=255)),
                ('description', models.CharField(default='', max_length=255)),
                ('image', models.ImageField(blank=True, null=True, upload_to='')),
                ('is_home', models.BooleanField(default=False)),
                ('is_visible', models.BooleanField(default=False)),
                ('comming', models.BooleanField(default=False)),
            ],
        ),
    ]
