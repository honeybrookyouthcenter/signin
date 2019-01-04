using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Microsoft.Graph;
using Microsoft.Toolkit.Services.MicrosoftGraph;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn
{
    class Graph
    {
        const string AppId = "6cd2d40f-e8d2-46be-ad3f-ac087d4b5284";
        readonly string[] Scopes = { "User.Read", "Contacts.ReadWrite" };

        public Graph()
        {
            MicrosoftGraphService instance = MicrosoftGraphService.Instance;
            instance.AuthenticationModel = MicrosoftGraphEnums.AuthenticationModel.V2;
            instance.Initialize(AppId,
                MicrosoftGraphEnums.ServicesToInitialize.UserProfile,
                Scopes);
        }

        public bool IsAuthenticated => MicrosoftGraphService.Instance.IsAuthenticated;

        public GraphServiceClient Provider => MicrosoftGraphService.Instance.GraphProvider;

        public async Task<bool> Login()
        {
            try
            {
                return await MicrosoftGraphService.Instance.TryLoginAsync();
            }
            catch (Exception ex)
            {
                await DataProvider.Current.ShowMessage("Could not log in.  Check your internet connection.", ex);
                return false;
            }
        }

        #region Contacts

        public async Task<IEnumerable<Contact>> GetContacts()
        {
            if (!IsAuthenticated)
            {
                await ((UwpDataProvider)DataProvider.Current).ShowAuthenticationMessage();
                return new List<Contact>();
            }

            var contacts = new List<Contact>();
            var currentRequest = Provider.Me.Contacts.Request().Select("id,givenName,surname,birthday,homeAddress");
            while (currentRequest != null)
            {
                var nextContacts = await currentRequest.GetAsync();
                contacts.AddRange(nextContacts);
                currentRequest = nextContacts.NextPageRequest;
            }

            return contacts;
        }

        public async Task<Contact> SaveContact(Contact contact)
        {
            try
            {
                if (!IsAuthenticated)
                    throw new InvalidOperationException("You must be logged in to save a contact!");

                return await Provider.Me.Contacts.Request().AddAsync(contact);
            }
            catch (Exception ex)
            {
                throw new Exception($"Contact save failed!", ex);
            }
        }

        #endregion
    }
}
