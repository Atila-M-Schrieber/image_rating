# Image Rating

This software provides a way to rate (jpg) images in a folder,
with ratings persistently stored in a csv file.

I built this tool to help me pick the best pictures from a pool of good pictures,
to post on social media, set as my profile picture, or print.

The rating is done with an [Élő rating system](https://en.wikipedia.org/wiki/Elo_rating_system).
The initial score for all images is 1200.

Images rated below a minimum score (1100 by default) will not be shown anymore,
to improve the speed of picking the right images.
The minimum score can be changed by setting the MIN_SCORE environmental variable.

The K-factor is 40 by default; it can be changed by setting the K environmental variable.
